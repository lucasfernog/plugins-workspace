// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Upload files from disk to a remote server over HTTP.
//!
//! Download files from a remote HTTP server to disk.
//!
//! ## Cargo features
//!
//! - **rustls-tls** *(enabled by default)*: Enables TLS functionality provided by `rustls`.
//! - **native-tls**: Enables TLS functionality provided by `native-tls`.
//! - **native-tls-vendored**: Enables the `vendored` feature of `native-tls`.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

mod transfer_stats;
use transfer_stats::TransferStats;

use futures_util::TryStreamExt;
use serde::{ser::Serializer, Deserialize, Serialize};
use tauri::{
    command,
    ipc::Channel,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Runtime,
};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::codec::{BytesCodec, FramedRead};

use read_progress_stream::ReadProgressStream;

use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

/// The HTTP method used to send the file in the `upload` command.
///
/// Serialized as an uppercase string (`"POST"`, `"PUT"` or `"PATCH"`) to match the JavaScript
/// guest bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// Send the file with an HTTP `POST` request. This is the default when no method is given.
    Post,
    /// Send the file with an HTTP `PUT` request.
    Put,
    /// Send the file with an HTTP `PATCH` request.
    Patch,
}

type Result<T> = std::result::Result<T, Error>;

/// The error type returned by this plugin's `upload` and `download` commands.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error, for example failing to open, read or write the file at the given path, or
    /// the background task that performs the transfer panicking.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An error returned by the underlying [`reqwest`] HTTP client while sending the request or
    /// reading the response.
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    /// The content length of the request or response could not be determined.
    #[error("{0}")]
    ContentLength(String),
    /// The HTTP response did not indicate success. Contains the status code and the response
    /// body text.
    #[error("request failed with status code {0}: {1}")]
    HttpErrorCode(u16, String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    progress: u64,
    progress_total: u64,
    total: u64,
    transfer_speed: u64,
}

#[command]
async fn download(
    url: String,
    file_path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    on_progress: Channel<ProgressPayload>,
) -> Result<()> {
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut request = if let Some(body) = body {
            client.post(&url).body(body)
        } else {
            client.get(&url)
        };
        // Loop trought the headers keys and values
        // and add them to the request object.
        for (key, value) in headers {
            request = request.header(&key, value);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(Error::HttpErrorCode(
                response.status().as_u16(),
                response.text().await.unwrap_or_default(),
            ));
        }
        let total = response.content_length().unwrap_or(0);

        // Stream into a temporary file next to the destination and only move it into place once
        // the whole body has been written, so a failed download neither leaves a partial file
        // behind nor destroys an existing file at `file_path`.
        let destination = download_destination(Path::new(&file_path)).await;
        let temp_path = temp_download_path(&destination)?;
        let file = File::create(&temp_path).await?;

        let result = async {
            let mut file = BufWriter::new(file);
            let mut stream = response.bytes_stream();

            let mut stats = TransferStats::default();
            while let Some(chunk) = stream.try_next().await? {
                file.write_all(&chunk).await?;
                stats.record_chunk_transfer(chunk.len());
                let _ = on_progress.send(ProgressPayload {
                    progress: chunk.len() as u64,
                    progress_total: stats.total_transferred,
                    total,
                    transfer_speed: stats.transfer_speed,
                });
            }
            file.flush().await?;
            // close the file before renaming it (required on Windows)
            drop(file);

            // keep the permissions of the file being replaced
            if let Ok(metadata) = tokio::fs::metadata(&destination).await {
                let _ = tokio::fs::set_permissions(&temp_path, metadata.permissions()).await;
            }
            tokio::fs::rename(&temp_path, &destination).await?;
            Ok::<(), Error>(())
        }
        .await;

        if result.is_err() {
            let _ = tokio::fs::remove_file(&temp_path).await;
        }
        result
    })
    .await
    .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
}

/// Resolves the path a download is finally written to. An existing destination is canonicalized,
/// so that downloading to a symlink keeps writing through it instead of replacing the link.
async fn download_destination(file_path: &Path) -> PathBuf {
    tokio::fs::canonicalize(file_path)
        .await
        .unwrap_or_else(|_| file_path.to_path_buf())
}

/// A unique, hidden temporary path in the same directory as `destination`, so it can be renamed
/// over the destination atomically.
fn temp_download_path(destination: &Path) -> std::io::Result<PathBuf> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let file_name = destination.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid download path: {}", destination.display()),
        )
    })?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or_default();
    let mut temp_name = OsString::from(".");
    temp_name.push(file_name);
    temp_name.push(format!(
        ".{}-{}-{nanos}.download",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    Ok(destination.with_file_name(temp_name))
}

#[command]
async fn upload(
    url: String,
    file_path: String,
    headers: HashMap<String, String>,
    method: Option<HttpMethod>,
    on_progress: Channel<ProgressPayload>,
) -> Result<String> {
    tokio::spawn(async move {
        // Read the file
        let file = File::open(&file_path).await?;
        let file_len = file.metadata().await.unwrap().len();

        // Get HTTP method (defaults to POST)
        let http_method = method.unwrap_or(HttpMethod::Post);

        // Create the request and attach the file to the body
        let client = reqwest::Client::new();
        let mut request = match http_method {
            HttpMethod::Put => client.put(&url),
            HttpMethod::Patch => client.patch(&url),
            HttpMethod::Post => client.post(&url),
        }
        .header(reqwest::header::CONTENT_LENGTH, file_len)
        .body(file_to_body(on_progress, file, file_len));

        // Loop through the headers keys and values
        // and add them to the request object.
        for (key, value) in headers {
            request = request.header(&key, value);
        }

        let response = request.send().await?;
        if response.status().is_success() {
            response.text().await.map_err(Into::into)
        } else {
            Err(Error::HttpErrorCode(
                response.status().as_u16(),
                response.text().await.unwrap_or_default(),
            ))
        }
    })
    .await
    .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
}

fn file_to_body(channel: Channel<ProgressPayload>, file: File, file_len: u64) -> reqwest::Body {
    let stream = FramedRead::new(file, BytesCodec::new()).map_ok(|r| r.freeze());

    let mut stats = TransferStats::default();
    reqwest::Body::wrap_stream(ReadProgressStream::new(
        stream,
        Box::new(move |progress, _total| {
            stats.record_chunk_transfer(progress as usize);
            let _ = channel.send(ProgressPayload {
                progress,
                progress_total: stats.total_transferred,
                total: file_len,
                transfer_speed: stats.transfer_speed,
            });
        }),
    ))
}

/// Initializes the upload plugin, registering the `upload` and `download` commands.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("upload")
        .invoke_handler(tauri::generate_handler![download, upload])
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{self, Mock, Server, ServerGuard};
    use tauri::ipc::InvokeResponseBody;
    struct MockedServer {
        _server: ServerGuard,
        url: String,
        mocked_endpoint: Mock,
    }

    #[tokio::test]
    async fn should_error_on_download_if_status_not_success() {
        let mocked_server = spawn_server_mocked(400).await;
        let result = download_file(mocked_server.url).await;
        mocked_server.mocked_endpoint.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_download_file_successfully() {
        let mocked_server = spawn_server_mocked(200).await;
        let result = download_file(mocked_server.url).await;
        mocked_server.mocked_endpoint.assert();
        assert!(
            result.is_ok(),
            "failed to download file: {}",
            result.unwrap_err()
        );
    }

    #[tokio::test]
    async fn should_error_on_upload_if_status_not_success() {
        let mocked_server = spawn_upload_server_mocked(500, "POST").await;
        let result = upload_file(mocked_server.url, None).await;
        mocked_server.mocked_endpoint.assert();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::HttpErrorCode(status, _) => assert_eq!(status, 500),
            _ => panic!("Expected HttpErrorCode error"),
        }
    }

    #[tokio::test]
    async fn should_error_on_upload_if_file_not_found() {
        let mocked_server = spawn_upload_server_mocked(200, "POST").await;
        let file_path = "/nonexistent/file.txt".to_string();
        let headers = HashMap::new();
        let sender: Channel<ProgressPayload> =
            Channel::new(|msg: InvokeResponseBody| -> tauri::Result<()> {
                let _ = msg;
                Ok(())
            });

        let result = upload(mocked_server.url, file_path, headers, None, sender).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Io(_) => {}
            _ => panic!("Expected IO error for missing file"),
        }
    }

    #[tokio::test]
    async fn should_upload_file_with_post_method() {
        let mocked_server = spawn_upload_server_mocked(200, "POST").await;
        let result = upload_file(mocked_server.url, Some(HttpMethod::Post)).await;
        mocked_server.mocked_endpoint.assert();
        assert!(
            result.is_ok(),
            "failed to upload file: {}",
            result.unwrap_err()
        );
        let response_body = result.unwrap();
        assert_eq!(response_body, "upload successful");
    }

    #[tokio::test]
    async fn should_upload_file_with_put_method() {
        let mocked_server = spawn_upload_server_mocked(200, "PUT").await;
        let result = upload_file(mocked_server.url, Some(HttpMethod::Put)).await;
        mocked_server.mocked_endpoint.assert();
        assert!(
            result.is_ok(),
            "failed to upload file with PUT: {}",
            result.unwrap_err()
        );
        let response_body = result.unwrap();
        assert_eq!(response_body, "upload successful");
    }

    #[tokio::test]
    async fn should_upload_file_with_patch_method() {
        let mocked_server = spawn_upload_server_mocked(200, "PATCH").await;
        let result = upload_file(mocked_server.url, Some(HttpMethod::Patch)).await;
        mocked_server.mocked_endpoint.assert();
        assert!(
            result.is_ok(),
            "failed to upload file with PATCH: {}",
            result.unwrap_err()
        );
        let response_body = result.unwrap();
        assert_eq!(response_body, "upload successful");
    }

    #[tokio::test]
    async fn failed_download_keeps_existing_file() {
        use tokio::io::AsyncReadExt;

        // A server that announces 100 bytes but closes the connection after 7.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let _ = socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\npartial")
                .await;
            let _ = socket.shutdown().await;
        });

        let dir = std::env::temp_dir().join(format!(
            "tauri-plugin-upload-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("existing.txt");
        std::fs::write(&file_path, "original contents").unwrap();

        let sender: Channel<ProgressPayload> =
            Channel::new(|_msg: InvokeResponseBody| -> tauri::Result<()> { Ok(()) });
        let result = download(
            url,
            file_path.to_string_lossy().into_owned(),
            HashMap::new(),
            None,
            sender,
        )
        .await;
        assert!(result.is_err(), "the truncated download must fail");

        assert_eq!(
            std::fs::read_to_string(&file_path).unwrap(),
            "original contents"
        );
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec![OsString::from("existing.txt")]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn temp_download_path_is_a_hidden_sibling() {
        let destination = Path::new("/some/dir/file.txt");
        let temp = temp_download_path(destination).unwrap();
        assert_eq!(temp.parent(), destination.parent());
        let name = temp.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with(".file.txt."), "{name}");
        assert!(name.ends_with(".download"), "{name}");
        assert_ne!(temp, temp_download_path(destination).unwrap());
        assert!(temp_download_path(Path::new("/")).is_err());
    }

    async fn download_file(url: String) -> Result<()> {
        let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test/download.txt").to_string();
        let headers = HashMap::new();
        let sender: Channel<ProgressPayload> =
            Channel::new(|msg: InvokeResponseBody| -> tauri::Result<()> {
                let _ = msg;
                Ok(())
            });
        download(url, file_path, headers, None, sender).await
    }

    async fn upload_file(url: String, method: Option<HttpMethod>) -> Result<String> {
        let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test/upload.txt").to_string();
        let headers = HashMap::new();
        let sender: Channel<ProgressPayload> =
            Channel::new(|msg: InvokeResponseBody| -> tauri::Result<()> {
                let _ = msg;
                Ok(())
            });
        upload(url, file_path, headers, method, sender).await
    }

    async fn spawn_server_mocked(return_status: usize) -> MockedServer {
        let mut _server = Server::new_async().await;
        let path = "/mock_test";
        let mock = _server
            .mock("GET", path)
            .with_status(return_status)
            .with_body("mocked response body")
            .create_async()
            .await;

        let url = _server.url() + path;
        MockedServer {
            _server,
            url,
            mocked_endpoint: mock,
        }
    }

    async fn spawn_upload_server_mocked(return_status: usize, method: &str) -> MockedServer {
        let mut _server = Server::new_async().await;
        let path = "/upload_test";
        let mock = _server
            .mock(method, path)
            .with_status(return_status)
            .with_body("upload successful")
            .match_header("content-length", "20")
            .create_async()
            .await;

        let url = _server.url() + path;
        MockedServer {
            _server,
            url,
            mocked_endpoint: mock,
        }
    }
}

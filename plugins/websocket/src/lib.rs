// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Open a WebSocket connection using a Rust client in JS.
//!
//! ## Cargo features
//!
//! - **rustls-tls** *(enabled by default)*: Enables TLS functionality provided by `rustls` with WebPKI roots.
//! - **rustls-tls-native-roots**: Enables TLS functionality provided by `rustls` with the platform's native certificate roots.
//! - **native-tls**: Enables TLS functionality provided by `native-tls`.
//! - **native-tls-vendored**: Enables the `vendored` feature of `native-tls`.
//!
//! At least one TLS feature is required for `wss://`; plain `ws://` works without one.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use http::header::{HeaderName, HeaderValue};
use serde::{ser::Serializer, Deserialize, Serialize};
use tauri::{
    ipc::Channel,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State, Window,
};
use tokio::{net::TcpStream, sync::Mutex};
#[cfg(any(
    feature = "rustls-tls",
    feature = "rustls-tls-native-roots",
    feature = "native-tls"
))]
use tokio_tungstenite::connect_async_tls_with_config;
#[cfg(not(any(
    feature = "rustls-tls",
    feature = "rustls-tls-native-roots",
    feature = "native-tls"
)))]
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::{
    tungstenite::{
        client::IntoClientRequest,
        protocol::{CloseFrame as ProtocolCloseFrame, WebSocketConfig},
        Message,
    },
    Connector, MaybeTlsStream, WebSocketStream,
};

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

type Id = u32;
type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketWriter = SplitSink<WebSocket, Message>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("connection not found for the given id: {0}")]
    ConnectionNotFound(Id),
    #[error(transparent)]
    InvalidHeaderValue(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue),
    #[error(transparent)]
    InvalidHeaderName(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderName),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

/// The writer half of every open connection.
///
/// The map lock is only held to look up, insert or remove an entry, never across network I/O;
/// each writer has its own lock, so a slow connection does not block sends on the others.
#[derive(Default)]
struct ConnectionManager(std::sync::Mutex<HashMap<Id, Arc<Mutex<WebSocketWriter>>>>);

impl ConnectionManager {
    fn connections(&self) -> std::sync::MutexGuard<'_, HashMap<Id, Arc<Mutex<WebSocketWriter>>>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Registers `writer` under a new random id that no open connection uses.
    fn insert(&self, writer: Arc<Mutex<WebSocketWriter>>) -> Id {
        insert_with_unique_id(&mut self.connections(), writer, rand::random)
    }

    /// Removes the connection `id` if it still refers to `writer`.
    fn remove(&self, id: Id, writer: &Arc<Mutex<WebSocketWriter>>) {
        let mut connections = self.connections();
        if connections
            .get(&id)
            .is_some_and(|current| Arc::ptr_eq(current, writer))
        {
            connections.remove(&id);
        }
    }
}

/// Inserts `value` under the first id produced by `next_id` that is not in use yet, so an
/// existing entry is never replaced.
fn insert_with_unique_id<T>(
    map: &mut HashMap<Id, T>,
    value: T,
    mut next_id: impl FnMut() -> Id,
) -> Id {
    loop {
        let id = next_id();
        if let std::collections::hash_map::Entry::Vacant(entry) = map.entry(id) {
            entry.insert(value);
            return id;
        }
    }
}

#[cfg(any(
    feature = "rustls-tls",
    feature = "rustls-tls-native-roots",
    feature = "native-tls"
))]
struct TlsConnector(Mutex<Option<Connector>>);

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum Max {
    None,
    Number(usize),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectionConfig {
    pub read_buffer_size: Option<usize>,
    pub write_buffer_size: Option<usize>,
    pub max_write_buffer_size: Option<usize>,
    pub max_message_size: Option<Max>,
    pub max_frame_size: Option<Max>,
    #[serde(default)]
    pub accept_unmasked_frames: bool,
    pub headers: Option<Vec<(String, String)>>,
}

impl From<ConnectionConfig> for WebSocketConfig {
    fn from(config: ConnectionConfig) -> Self {
        let mut builder =
            WebSocketConfig::default().accept_unmasked_frames(config.accept_unmasked_frames);

        if let Some(read_buffer_size) = config.read_buffer_size {
            builder = builder.read_buffer_size(read_buffer_size)
        }

        if let Some(write_buffer_size) = config.write_buffer_size {
            builder = builder.write_buffer_size(write_buffer_size)
        }

        if let Some(max_write_buffer_size) = config.max_write_buffer_size {
            builder = builder.max_write_buffer_size(max_write_buffer_size)
        }

        if let Some(max_message_size) = config.max_message_size {
            let max_size = match max_message_size {
                Max::None => Option::None,
                Max::Number(n) => Some(n),
            };
            builder = builder.max_message_size(max_size);
        }

        if let Some(max_frame_size) = config.max_frame_size {
            let max_size = match max_frame_size {
                Max::None => Option::None,
                Max::Number(n) => Some(n),
            };
            builder = builder.max_frame_size(max_size);
        }

        builder
    }
}

#[derive(Deserialize, Serialize)]
struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}

#[tauri::command]
async fn connect<R: Runtime>(
    window: Window<R>,
    url: String,
    on_message: Channel<serde_json::Value>,
    config: Option<ConnectionConfig>,
) -> Result<Id> {
    let mut request = url.into_client_request()?;

    if let Some(headers) = config.as_ref().and_then(|c| c.headers.as_ref()) {
        for (k, v) in headers {
            let header_name = HeaderName::from_str(k.as_str())?;
            let header_value = HeaderValue::from_str(v.as_str())?;
            request.headers_mut().insert(header_name, header_value);
        }
    }

    #[cfg(any(
        feature = "rustls-tls",
        feature = "rustls-tls-native-roots",
        feature = "native-tls"
    ))]
    let tls_connector = match window.try_state::<TlsConnector>() {
        Some(tls_connector) => tls_connector.0.lock().await.clone(),
        None => None,
    };

    #[cfg(any(
        feature = "rustls-tls",
        feature = "rustls-tls-native-roots",
        feature = "native-tls"
    ))]
    let (ws_stream, _) =
        connect_async_tls_with_config(request, config.map(Into::into), false, tls_connector)
            .await?;
    #[cfg(not(any(
        feature = "rustls-tls",
        feature = "rustls-tls-native-roots",
        feature = "native-tls"
    )))]
    let (ws_stream, _) = connect_async_with_config(request, config.map(Into::into), false).await?;

    // Register the writer before resolving, so a `send` issued right after `connect` finds it.
    let (write, mut read) = ws_stream.split();
    let writer = Arc::new(Mutex::new(write));
    let id = window.state::<ConnectionManager>().insert(writer.clone());

    tauri::async_runtime::spawn(async move {
        while let Some(message) = read.next().await {
            if let Ok(Message::Close(_)) = message {
                window.state::<ConnectionManager>().remove(id, &writer);
            }

            let response = match message {
                Ok(Message::Text(t)) => {
                    serde_json::to_value(WebSocketMessage::Text(t.to_string())).unwrap()
                }
                Ok(Message::Binary(t)) => {
                    serde_json::to_value(WebSocketMessage::Binary(t.to_vec())).unwrap()
                }
                Ok(Message::Ping(t)) => {
                    serde_json::to_value(WebSocketMessage::Ping(t.to_vec())).unwrap()
                }
                Ok(Message::Pong(t)) => {
                    serde_json::to_value(WebSocketMessage::Pong(t.to_vec())).unwrap()
                }
                Ok(Message::Close(t)) => {
                    serde_json::to_value(WebSocketMessage::Close(t.map(|v| CloseFrame {
                        code: v.code.into(),
                        reason: v.reason.to_string(),
                    })))
                    .unwrap()
                }
                Ok(Message::Frame(_)) => serde_json::Value::Null, // This value can't be recieved.
                Err(e) => serde_json::to_value(Error::from(e)).unwrap(),
            };

            let _ = on_message.send(response);
        }

        // The stream has ended (close handshake, error, or the peer going away without a Close
        // frame): the connection can no longer be used, so forget it.
        window.state::<ConnectionManager>().remove(id, &writer);
    });

    Ok(id)
}

#[tauri::command]
async fn send(
    manager: State<'_, ConnectionManager>,
    id: Id,
    message: WebSocketMessage,
) -> Result<()> {
    // Clone the writer out so the map lock is not held while sending.
    let writer = manager.connections().get(&id).cloned();
    if let Some(writer) = writer {
        writer
            .lock()
            .await
            .send(match message {
                WebSocketMessage::Text(t) => Message::Text(t.into()),
                WebSocketMessage::Binary(t) => Message::Binary(t.into()),
                WebSocketMessage::Ping(t) => Message::Ping(t.into()),
                WebSocketMessage::Pong(t) => Message::Pong(t.into()),
                WebSocketMessage::Close(t) => Message::Close(t.map(|v| ProtocolCloseFrame {
                    code: v.code.into(),
                    reason: v.reason.into(),
                })),
            })
            .await?;
        Ok(())
    } else {
        Err(Error::ConnectionNotFound(id))
    }
}

/// Initializes the plugin with the default [`Builder`], i.e. without a custom TLS [`Connector`].
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::default().build()
}

/// Builder for the WebSocket plugin, used to configure a custom TLS [`Connector`] before calling [`Builder::build`].
#[derive(Default)]
pub struct Builder {
    tls_connector: Option<Connector>,
}

impl Builder {
    /// Creates a new [`Builder`] with no custom TLS [`Connector`] configured.
    pub fn new() -> Self {
        Self {
            tls_connector: None,
        }
    }

    /// Sets the TLS [`Connector`] used to establish `wss://` connections.
    ///
    /// When this is not called (or is called with [`Connector::Plain`]) and a `rustls-tls` or
    /// `rustls-tls-native-roots` feature is enabled, [`Builder::build`] installs `rustls`'s `ring`
    /// crypto provider as the process default if none is installed yet.
    pub fn tls_connector(mut self, connector: Connector) -> Self {
        self.tls_connector.replace(connector);
        self
    }

    /// Builds the plugin, registering the `connect` and `send` commands and the shared connection state.
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        PluginBuilder::new("websocket")
            .invoke_handler(tauri::generate_handler![connect, send])
            .setup(|app, _api| {
                #[cfg(any(feature = "rustls-tls", feature = "rustls-tls-native-roots"))]
                if (self.tls_connector.is_none()
                    || matches!(self.tls_connector, Some(Connector::Plain)))
                    && rustls::crypto::CryptoProvider::get_default().is_none()
                {
                    // This can only fail if there is already a default provider which we checked for already.
                    let _ = rustls::crypto::ring::default_provider().install_default();
                }

                app.manage(ConnectionManager::default());
                #[cfg(any(
                    feature = "rustls-tls",
                    feature = "rustls-tls-native-roots",
                    feature = "native-tls"
                ))]
                app.manage(TlsConnector(Mutex::new(self.tls_connector)));
                Ok(())
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_id_skips_ids_in_use() {
        let mut map = HashMap::from([(1, "first"), (2, "second")]);
        let mut candidates = [1, 2, 3].into_iter();
        let id = insert_with_unique_id(&mut map, "third", || candidates.next().unwrap());
        assert_eq!(id, 3);
        assert_eq!(map[&1], "first");
        assert_eq!(map[&2], "second");
        assert_eq!(map[&3], "third");
    }
}

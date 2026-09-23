// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Error, ErrorKind, Write},
    os::unix::{
        fs::OpenOptionsExt,
        io::AsRawFd,
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;
use crate::SingleInstanceCallback;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Config, Manager, RunEvent, Runtime,
};
use tokio::io::AsyncReadExt;

/// How long a starting instance waits for the instance holding the lock to start listening.
const FIND_FIRST_INSTANCE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long the first instance waits for another instance to send its whole message.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Largest message accepted from another instance (4 times macOS' `ARG_MAX`).
const MAX_MESSAGE_SIZE: u64 = 4 * 1024 * 1024;

pub fn init<R: Runtime>(cb: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app, _api| {
            let socket = socket_path(app.config(), app.package_info());

            match claim_or_notify(&socket) {
                Ok(Claim::Notified) => {
                    std::process::exit(0);
                }
                Ok(Claim::Primary { listener, lock }) => {
                    app.manage(OwnedSocket(Mutex::new(Some((socket, lock)))));
                    listen_for_other_instances(listener, app.clone(), cb);
                }
                Err(e) => {
                    tracing::debug!("single_instance failed to notify - launching normally: {e}");
                }
            }
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                destroy(app);
            }
        })
        .build()
}

/// The socket this instance listens on, and the lock that makes it the first instance. Released
/// by [`destroy`].
struct OwnedSocket(Mutex<Option<(PathBuf, File)>>);

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    // Only remove the socket if this instance created it, and only once: by the time `destroy`
    // runs a second time, a new instance may have created a socket at the same path.
    if let Some((socket, lock)) = manager
        .try_state::<OwnedSocket>()
        .and_then(|socket| socket.0.lock().unwrap().take())
    {
        socket_cleanup(&socket);
        // Remove the socket before releasing the lock, so the next instance doesn't connect to a
        // socket that is going away.
        drop(lock);
    }
}

fn socket_path(config: &Config, _package_info: &tauri::PackageInfo) -> PathBuf {
    let identifier = config.identifier.replace(['.', '-'].as_ref(), "_");

    #[cfg(feature = "semver")]
    let identifier = format!(
        "{identifier}_{}",
        semver_compat_string(&_package_info.version),
    );

    // Use /tmp as socket path must be shorter than 100 chars.
    PathBuf::from(format!("/tmp/{}_si.sock", identifier))
}

fn socket_cleanup(socket: &PathBuf) {
    let _ = std::fs::remove_file(socket);
}

enum Claim {
    /// Another instance is running and received our arguments.
    Notified,
    /// This is the first instance.
    Primary { listener: UnixListener, lock: File },
}

/// Either notifies the running instance or makes this process the first instance.
///
/// The first instance holds an exclusive `flock` on `<socket>.lock` for its whole lifetime (the
/// OS releases it if the process dies), so two instances starting at the same time can't both
/// remove the socket and bind their own: only the one that gets the lock binds, the other one
/// waits for it to listen and notifies it.
fn claim_or_notify(socket: &PathBuf) -> Result<Claim, Error> {
    // Fast path, and compatibility with first instances built with older plugin versions, which
    // don't take the lock.
    let connect_error = match notify_singleton(socket) {
        Ok(()) => return Ok(Claim::Notified),
        Err(e) => e,
    };
    if !matches!(
        connect_error.kind(),
        ErrorKind::NotFound | ErrorKind::ConnectionRefused
    ) {
        return Err(connect_error);
    }

    let lock_path = lock_path(socket);
    let deadline = Instant::now() + FIND_FIRST_INSTANCE_TIMEOUT;
    loop {
        if let Some(lock) = try_lock(&lock_path)? {
            // Any socket file left is stale: its owner died without cleaning up.
            socket_cleanup(socket);
            let listener = UnixListener::bind(socket)?;
            return Ok(Claim::Primary { listener, lock });
        }

        // Another instance holds the lock: it is starting up (and about to listen) or running.
        match notify_singleton(socket) {
            Ok(()) => return Ok(Claim::Notified),
            Err(e) if Instant::now() >= deadline => return Err(e),
            Err(_) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

fn lock_path(socket: &Path) -> PathBuf {
    let mut path = socket.as_os_str().to_owned();
    path.push(".lock");
    PathBuf::from(path)
}

/// Takes an exclusive, non-blocking `flock` on `path`. Returns `None` if another process holds
/// it.
fn try_lock(path: &Path) -> Result<Option<File>, Error> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        Ok(Some(file))
    } else {
        let error = Error::last_os_error();
        if error.kind() == ErrorKind::WouldBlock {
            Ok(None)
        } else {
            Err(error)
        }
    }
}

fn notify_singleton(socket: &PathBuf) -> Result<(), Error> {
    let stream = UnixStream::connect(socket)?;
    let mut bf = BufWriter::new(&stream);
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    bf.write_all(cwd.as_bytes())?;
    bf.write_all(b"\0\0")?;
    let args_joined = std::env::args().collect::<Vec<String>>().join("\0");
    bf.write_all(args_joined.as_bytes())?;
    bf.flush()?;
    drop(bf);
    Ok(())
}

fn listen_for_other_instances<A: Runtime>(
    listener: UnixListener,
    app: AppHandle<A>,
    cb: Box<SingleInstanceCallback<A>>,
) {
    let cb = Arc::new(Mutex::new(cb));
    tauri::async_runtime::spawn(async move {
        let listener = listener
            .set_nonblocking(true)
            .and_then(|()| tokio::net::UnixListener::from_std(listener));
        match listener {
            Ok(listener) => loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        // Read each connection in its own task, so a client that never closes
                        // the connection can't block the notifications of later instances.
                        let app = app.clone();
                        let cb = cb.clone();
                        tauri::async_runtime::spawn(async move {
                            match read_message(stream).await {
                                Ok((cwd, args)) => {
                                    let mut cb = cb.lock().unwrap_or_else(PoisonError::into_inner);
                                    cb(&app, args, cwd);
                                }
                                Err(e) => {
                                    tracing::debug!("single_instance failed to be notified: {e}")
                                }
                            }
                        });
                    }
                    Err(err) => {
                        tracing::debug!("single_instance failed to be notified: {}", err);
                        continue;
                    }
                }
            },
            Err(err) => {
                tracing::error!(
                    "single_instance failed to listen to other processes - launching normally: {}",
                    err
                );
            }
        }
    });
}

/// Reads the `cwd\0\0arg0\0arg1…` message another instance sends, with a time and size limit.
async fn read_message(stream: tokio::net::UnixStream) -> Result<(String, Vec<String>), Error> {
    let mut data = Vec::new();
    tokio::time::timeout(
        READ_TIMEOUT,
        stream.take(MAX_MESSAGE_SIZE + 1).read_to_end(&mut data),
    )
    .await
    .map_err(|_| Error::new(ErrorKind::TimedOut, "timed out reading the message"))??;
    if data.len() as u64 > MAX_MESSAGE_SIZE {
        return Err(Error::new(ErrorKind::InvalidData, "message too large"));
    }
    Ok(decode_message(&data))
}

/// Decodes the `cwd\0\0arg0\0arg1…` message. Invalid UTF-8 is converted lossily.
fn decode_message(data: &[u8]) -> (String, Vec<String>) {
    let data = String::from_utf8_lossy(data);
    let (cwd, args) = data.split_once("\0\0").unwrap_or_default();
    (
        cwd.to_string(),
        args.split('\0').map(String::from).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn temp_socket(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("si-test-{}-{name}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        path
    }

    fn cleanup(socket: &PathBuf, lock: File) {
        socket_cleanup(socket);
        drop(lock);
        let _ = std::fs::remove_file(lock_path(socket));
    }

    #[test]
    fn second_claim_notifies_the_first() {
        let socket = temp_socket("notify");
        let Claim::Primary { listener, lock } = claim_or_notify(&socket).unwrap() else {
            panic!("expected to become the first instance");
        };
        assert!(matches!(claim_or_notify(&socket), Ok(Claim::Notified)));

        let (mut stream, _) = listener.accept().unwrap();
        let mut data = Vec::new();
        stream.read_to_end(&mut data).unwrap();
        assert!(data.windows(2).any(|w| w == b"\0\0"));

        cleanup(&socket, lock);
    }

    #[test]
    fn waits_for_a_starting_first_instance() {
        let socket = temp_socket("starting");
        // the first instance took the lock but doesn't listen yet
        let lock = try_lock(&lock_path(&socket)).unwrap().unwrap();
        let first = {
            let socket = socket.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(300));
                let listener = UnixListener::bind(&socket).unwrap();
                let (mut stream, _) = listener.accept().unwrap();
                stream.read_to_end(&mut Vec::new()).unwrap();
            })
        };
        assert!(matches!(claim_or_notify(&socket), Ok(Claim::Notified)));
        first.join().unwrap();
        cleanup(&socket, lock);
    }

    #[test]
    fn stale_socket_is_replaced() {
        let socket = temp_socket("stale");
        // a socket file nobody listens on anymore
        drop(UnixListener::bind(&socket).unwrap());
        assert!(socket.exists());

        let Claim::Primary { lock, .. } = claim_or_notify(&socket).unwrap() else {
            panic!("expected to become the first instance");
        };
        cleanup(&socket, lock);
    }

    #[test]
    fn decodes_messages() {
        assert_eq!(
            decode_message(b"/cwd\0\0/app\0--flag\0a|b"),
            (
                "/cwd".to_string(),
                vec!["/app".into(), "--flag".into(), "a|b".into()]
            )
        );
        assert_eq!(
            decode_message(b"/c\xffwd\0\0a\xfe"),
            ("/c\u{fffd}wd".to_string(), vec!["a\u{fffd}".into()])
        );
        assert_eq!(decode_message(b""), (String::new(), vec![String::new()]));
    }

    #[test]
    fn reads_concurrent_and_invalid_messages() {
        tauri::async_runtime::block_on(async {
            // a client that never sends anything nor closes doesn't prevent reading another one
            let (_idle, idle_peer) = UnixStream::pair().unwrap();
            let (mut client, peer) = UnixStream::pair().unwrap();
            idle_peer.set_nonblocking(true).unwrap();
            peer.set_nonblocking(true).unwrap();
            let idle = tauri::async_runtime::spawn(read_message(
                tokio::net::UnixStream::from_std(idle_peer).unwrap(),
            ));

            client.write_all(b"/cwd\0\0/app\0\xff").unwrap();
            drop(client);
            let message = read_message(tokio::net::UnixStream::from_std(peer).unwrap())
                .await
                .unwrap();
            assert_eq!(
                message,
                ("/cwd".to_string(), vec!["/app".into(), "\u{fffd}".into()])
            );
            idle.abort();
        });
    }

    #[test]
    fn lock_is_exclusive_until_released() {
        let socket = temp_socket("lock");
        let lock_path = lock_path(&socket);
        let lock = try_lock(&lock_path).unwrap().expect("lock is free");
        assert!(try_lock(&lock_path).unwrap().is_none());
        drop(lock);
        assert!(try_lock(&lock_path).unwrap().is_some());
        let _ = std::fs::remove_file(lock_path);
    }
}

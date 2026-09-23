// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, path::PathBuf, string::FromUtf8Error};

use encoding_rs::Encoding;
use serde::{Deserialize, Serialize};
use tauri::{
    ipc::{Channel, CommandScope, GlobalScope},
    Manager, Runtime, State, Window,
};

#[allow(deprecated)]
use crate::open::Program;
use crate::{
    process::{CommandEvent, TerminatedPayload},
    scope::ExecuteArgs,
    Shell,
};

type ChildId = u32;

/// How long to wait before sending a child process event to the webview again.
const SEND_EVENT_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(15);
/// How many times sending an event is retried before giving up (about 5 seconds).
const SEND_EVENT_MAX_RETRIES: u32 = 333;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "payload")]
#[non_exhaustive]
pub enum JSCommandEvent {
    /// Stderr bytes until a newline (\n) or carriage return (\r) is found.
    Stderr(Buffer),
    /// Stdout bytes until a newline (\n) or carriage return (\r) is found.
    Stdout(Buffer),
    /// An error happened waiting for the command to finish or converting the stdout/stderr bytes to an UTF-8 string.
    Error(String),
    /// Command process terminated.
    Terminated(TerminatedPayload),
}

fn get_event_buffer(line: Vec<u8>, encoding: EncodingWrapper) -> Result<Buffer, FromUtf8Error> {
    match encoding {
        EncodingWrapper::Text(character_encoding) => match character_encoding {
            Some(encoding) => Ok(Buffer::Text(
                encoding.decode_with_bom_removal(&line).0.into(),
            )),
            None => String::from_utf8(line).map(Buffer::Text),
        },
        EncodingWrapper::Raw => Ok(Buffer::Raw(line)),
    }
}

/// Whether lines of this encoding cannot be split on the `\n` and `\r` bytes, because
/// those bytes also appear inside other characters (UTF-16 encodes `\n` as two bytes).
fn needs_line_decoder(encoding: &'static Encoding) -> bool {
    encoding == encoding_rs::UTF_16LE || encoding == encoding_rs::UTF_16BE
}

/// Decodes a byte stream incrementally and splits the decoded text into lines, each
/// ending with its `\n`, `\r\n` or `\r` delimiter like the lines read for UTF-8 output.
struct LineDecoder {
    decoder: encoding_rs::Decoder,
    pending: String,
}

impl LineDecoder {
    fn new(encoding: &'static Encoding) -> Self {
        Self {
            decoder: encoding.new_decoder_with_bom_removal(),
            pending: String::new(),
        }
    }

    /// Decodes the next chunk and returns the lines it completed.
    /// With `last`, the stream ended and whatever is left is returned as the last line.
    fn decode(&mut self, bytes: &[u8], last: bool) -> Vec<String> {
        let mut input = bytes;
        loop {
            let capacity = self
                .decoder
                .max_utf8_buffer_length(input.len())
                .unwrap_or(input.len() * 3)
                .max(4);
            self.pending.reserve(capacity);
            let (result, read, _) = self
                .decoder
                .decode_to_string(input, &mut self.pending, last);
            input = &input[read..];
            if result == encoding_rs::CoderResult::InputEmpty {
                break;
            }
        }

        let mut lines = Vec::new();
        let mut start = 0;
        let bytes = self.pending.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let end = match bytes[i] {
                b'\n' => Some(i + 1),
                b'\r' => match bytes.get(i + 1) {
                    Some(b'\n') => Some(i + 2),
                    Some(_) => Some(i + 1),
                    // a `\r` at the end may still be followed by a `\n`
                    None if last => Some(i + 1),
                    None => None,
                },
                _ => None,
            };
            if let Some(end) = end {
                lines.push(self.pending[start..end].to_string());
                start = end;
                i = end;
            } else {
                i += 1;
            }
        }
        self.pending.drain(..start);
        if last && !self.pending.is_empty() {
            lines.push(std::mem::take(&mut self.pending));
        }
        lines
    }
}

impl JSCommandEvent {
    pub fn new(event: CommandEvent, encoding: EncodingWrapper) -> Self {
        match event {
            CommandEvent::Terminated(payload) => JSCommandEvent::Terminated(payload),
            CommandEvent::Error(error) => JSCommandEvent::Error(error),
            CommandEvent::Stderr(line) => get_event_buffer(line, encoding)
                .map(JSCommandEvent::Stderr)
                .unwrap_or_else(|e| JSCommandEvent::Error(e.to_string())),
            CommandEvent::Stdout(line) => get_event_buffer(line, encoding)
                .map(JSCommandEvent::Stdout)
                .unwrap_or_else(|e| JSCommandEvent::Error(e.to_string())),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum Buffer {
    Text(String),
    Raw(Vec<u8>),
}

#[derive(Debug, Copy, Clone)]
pub enum EncodingWrapper {
    Raw,
    Text(Option<&'static Encoding>),
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOptions {
    #[serde(default)]
    sidecar: bool,
    cwd: Option<PathBuf>,
    // by default we don't add any env variables to the spawned process
    // but the env is an `Option` so when it's `None` we clear the env.
    #[serde(default = "default_env")]
    env: Option<HashMap<String, String>>,
    // Character encoding for stdout/stderr
    encoding: Option<String>,
}

#[allow(clippy::unnecessary_wraps)]
fn default_env() -> Option<HashMap<String, String>> {
    Some(HashMap::default())
}

#[inline(always)]
fn prepare_cmd<R: Runtime>(
    window: Window<R>,
    program: String,
    args: ExecuteArgs,
    options: CommandOptions,
    command_scope: CommandScope<crate::scope::ScopeAllowedCommand>,
    global_scope: GlobalScope<crate::scope::ScopeAllowedCommand>,
) -> crate::Result<(crate::process::Command, EncodingWrapper)> {
    let scope = crate::scope::ShellScope {
        scopes: command_scope
            .allows()
            .iter()
            .chain(global_scope.allows())
            .collect(),
    };

    let mut command = if options.sidecar {
        let program = PathBuf::from(program);
        let program_as_string = program.display().to_string();
        let has_extension = program.extension().is_some_and(|ext| ext == "exe");
        let program_no_ext_as_string = if has_extension {
            program.with_extension("").display().to_string()
        } else {
            program_as_string.clone()
        };
        let configured_sidecar = window
            .config()
            .bundle
            .external_bin
            .as_ref()
            .and_then(|bins| {
                bins.iter()
                    .find(|b| b == &&program_as_string || b == &&program_no_ext_as_string)
            })
            .cloned();
        if let Some(sidecar) = configured_sidecar {
            scope.prepare_sidecar(&program.to_string_lossy(), &sidecar, args)?
        } else {
            return Err(crate::Error::SidecarNotAllowed(program));
        }
    } else {
        match scope.prepare(&program, args) {
            Ok(cmd) => cmd,
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("{e}");
                return Err(crate::Error::ProgramNotAllowed(PathBuf::from(program)));
            }
        }
    };
    if let Some(cwd) = options.cwd {
        command = command.current_dir(cwd);
    }
    if let Some(env) = options.env {
        command = command.envs(env);
    } else {
        command = command.env_clear();
    }

    let encoding = match options.encoding {
        Option::None => EncodingWrapper::Text(None),
        Some(encoding) => match encoding.as_str() {
            "raw" => {
                command = command.set_raw_out(true);
                EncodingWrapper::Raw
            }
            _ => {
                if let Some(text_encoding) = Encoding::for_label(encoding.as_bytes()) {
                    if needs_line_decoder(text_encoding) {
                        // read the output as it comes and split it into lines once decoded
                        command = command.set_raw_out(true);
                    }
                    EncodingWrapper::Text(Some(text_encoding))
                } else {
                    return Err(crate::Error::UnknownEncoding(encoding));
                }
            }
        },
    };

    Ok((command, encoding))
}

#[derive(Serialize)]
#[serde(untagged)]
enum Output {
    String(String),
    Raw(Vec<u8>),
}

#[derive(Serialize)]
pub struct ChildProcessReturn {
    code: Option<i32>,
    signal: Option<i32>,
    stdout: Output,
    stderr: Output,
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn execute<R: Runtime>(
    window: Window<R>,
    program: String,
    args: ExecuteArgs,
    options: CommandOptions,
    command_scope: CommandScope<crate::scope::ScopeAllowedCommand>,
    global_scope: GlobalScope<crate::scope::ScopeAllowedCommand>,
) -> crate::Result<ChildProcessReturn> {
    let (command, encoding) =
        prepare_cmd(window, program, args, options, command_scope, global_scope)?;

    let mut command: std::process::Command = command.into();
    let output = command.output()?;

    let (stdout, stderr) = match encoding {
        EncodingWrapper::Text(Some(encoding)) => (
            Output::String(encoding.decode_with_bom_removal(&output.stdout).0.into()),
            Output::String(encoding.decode_with_bom_removal(&output.stderr).0.into()),
        ),
        EncodingWrapper::Text(None) => (
            Output::String(String::from_utf8(output.stdout)?),
            Output::String(String::from_utf8(output.stderr)?),
        ),
        EncodingWrapper::Raw => (Output::Raw(output.stdout), Output::Raw(output.stderr)),
    };

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    Ok(ChildProcessReturn {
        code: output.status.code(),
        #[cfg(windows)]
        signal: None,
        #[cfg(unix)]
        signal: output.status.signal(),
        stdout,
        stderr,
    })
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn spawn<R: Runtime>(
    window: Window<R>,
    shell: State<'_, Shell<R>>,
    program: String,
    args: ExecuteArgs,
    on_event: Channel<JSCommandEvent>,
    options: CommandOptions,
    command_scope: CommandScope<crate::scope::ScopeAllowedCommand>,
    global_scope: GlobalScope<crate::scope::ScopeAllowedCommand>,
) -> crate::Result<ChildId> {
    let (command, encoding) =
        prepare_cmd(window, program, args, options, command_scope, global_scope)?;

    let (mut rx, child) = command.spawn()?;

    let pid = child.pid();
    shell.children.lock().unwrap().insert(pid, child);
    let children = shell.children.clone();

    tauri::async_runtime::spawn(async move {
        // Sending fails while the webview cannot receive the message yet (see #1298), so
        // it is retried for a while. If it keeps failing the webview is most likely gone:
        // stop forwarding, but keep draining the events so the child does not block on
        // a full stdout/stderr pipe.
        let mut forwarding = true;
        let mut line_decoders = match encoding {
            EncodingWrapper::Text(Some(encoding)) if needs_line_decoder(encoding) => {
                Some((LineDecoder::new(encoding), LineDecoder::new(encoding)))
            }
            _ => None,
        };
        while let Some(event) = rx.recv().await {
            if matches!(event, CommandEvent::Terminated(_)) {
                children.lock().unwrap().remove(&pid);
            };
            if !forwarding {
                continue;
            }
            let js_events = match (&mut line_decoders, event) {
                (Some((stdout, _)), CommandEvent::Stdout(bytes)) => stdout
                    .decode(&bytes, false)
                    .into_iter()
                    .map(|line| JSCommandEvent::Stdout(Buffer::Text(line)))
                    .collect(),
                (Some((_, stderr)), CommandEvent::Stderr(bytes)) => stderr
                    .decode(&bytes, false)
                    .into_iter()
                    .map(|line| JSCommandEvent::Stderr(Buffer::Text(line)))
                    .collect(),
                // both streams are closed once the process terminated
                (Some((stdout, stderr)), event @ CommandEvent::Terminated(_)) => stdout
                    .decode(&[], true)
                    .into_iter()
                    .map(|line| JSCommandEvent::Stdout(Buffer::Text(line)))
                    .chain(
                        stderr
                            .decode(&[], true)
                            .into_iter()
                            .map(|line| JSCommandEvent::Stderr(Buffer::Text(line))),
                    )
                    .chain(std::iter::once(JSCommandEvent::new(event, encoding)))
                    .collect(),
                (_, event) => vec![JSCommandEvent::new(event, encoding)],
            };

            for js_event in js_events {
                let mut retries = 0;
                while on_event.send(js_event.clone()).is_err() {
                    if retries == SEND_EVENT_MAX_RETRIES {
                        log::warn!(
                            "failed to send the events of process {pid} to the webview, giving up"
                        );
                        forwarding = false;
                        break;
                    }
                    retries += 1;
                    tokio::time::sleep(SEND_EVENT_RETRY_DELAY).await;
                }
                if !forwarding {
                    break;
                }
            }
        }
    });

    Ok(pid)
}

#[tauri::command]
pub fn stdin_write<R: Runtime>(
    _window: Window<R>,
    shell: State<'_, Shell<R>>,
    pid: ChildId,
    buffer: Buffer,
) -> crate::Result<()> {
    if let Some(child) = shell.children.lock().unwrap().get_mut(&pid) {
        match buffer {
            Buffer::Text(t) => child.write(t.as_bytes())?,
            Buffer::Raw(r) => child.write(&r)?,
        }
    }
    Ok(())
}

#[tauri::command]
pub fn kill<R: Runtime>(
    _window: Window<R>,
    shell: State<'_, Shell<R>>,
    pid: ChildId,
) -> crate::Result<()> {
    if let Some(child) = shell.children.lock().unwrap().remove(&pid) {
        child.kill()?;
    }
    Ok(())
}

#[allow(deprecated)]
#[tauri::command]
pub async fn open<R: Runtime>(
    _window: Window<R>,
    shell: State<'_, Shell<R>>,
    path: String,
    with: Option<Program>,
) -> crate::Result<()> {
    crate::open::open(Some(&shell.open_scope), path, with)
}

#[cfg(test)]
mod tests {
    use super::LineDecoder;

    fn utf16le(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    #[test]
    fn utf16_lines_are_split_after_decoding() {
        let mut decoder = LineDecoder::new(encoding_rs::UTF_16LE);
        let bytes = utf16le("one\ntwo\r\nthree\rfour");
        // split in the middle of the two bytes of the first `\n`, as the pipe may
        let split = 7;
        assert_eq!(decoder.decode(&bytes[..split], false), Vec::<String>::new());
        assert_eq!(
            decoder.decode(&bytes[split..], false),
            vec!["one\n", "two\r\n", "three\r"]
        );
        assert_eq!(decoder.decode(&[], true), vec!["four"]);
    }

    #[test]
    fn utf16_trailing_carriage_return_waits_for_a_newline() {
        let mut decoder = LineDecoder::new(encoding_rs::UTF_16BE);
        let bytes: Vec<u8> = "a\r\n".encode_utf16().flat_map(u16::to_be_bytes).collect();
        assert_eq!(decoder.decode(&bytes[..4], false), Vec::<String>::new());
        assert_eq!(decoder.decode(&bytes[4..], false), vec!["a\r\n"]);
        assert_eq!(decoder.decode(&[], true), Vec::<String>::new());
    }
}

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Upload files from disk to a remote server over HTTP, and download files from a remote HTTP
 * server to disk.
 *
 * @module
 */

import { invoke, Channel } from '@tauri-apps/api/core'

/**
 * The payload sent to a {@link ProgressHandler} while an upload or download is in progress.
 */
interface ProgressPayload {
  /**
   * The number of bytes transferred since the previous progress event (i.e. the size of the
   * last chunk), not the cumulative total.
   */
  progress: number
  /**
   * The cumulative number of bytes transferred so far.
   */
  progressTotal: number
  /**
   * The total size of the transfer in bytes. For a download this is the value of the response's
   * `Content-Length` header, and is `0` if the server did not send it or the response body is
   * compressed. For an upload this is the size of the file being uploaded.
   */
  total: number
  /**
   * The current transfer speed, approximately in bytes per second. It is recalculated about
   * every 500 milliseconds and stays at `0` until then.
   */
  transferSpeed: number
}

/**
 * A callback invoked with a {@link ProgressPayload} every time a chunk of data is
 * uploaded or downloaded.
 */
type ProgressHandler = (progress: ProgressPayload) => void

/**
 * The HTTP method used to send the file to the server in {@link upload}.
 */
enum HttpMethod {
  /**
   * Send the file using an HTTP `POST` request. This is the default when no method is given.
   */
  Post = 'POST',
  /**
   * Send the file using an HTTP `PUT` request.
   */
  Put = 'PUT',
  /**
   * Send the file using an HTTP `PATCH` request.
   */
  Patch = 'PATCH'
}

/**
 * Uploads a file at the given path to a URL, using the file's contents as the request body.
 *
 * @example
 * ```typescript
 * import { upload } from '@tauri-apps/plugin-upload';
 * import { appDataDir, join } from '@tauri-apps/api/path';
 *
 * const response = await upload(
 *   'https://example.com/file-upload',
 *   await join(await appDataDir(), 'file.txt'),
 *   ({ progressTotal, total }) => console.log(`Uploaded ${progressTotal} of ${total} bytes`),
 *   new Map([['Authorization', 'Bearer <token>']])
 * );
 * ```
 *
 * @param url The URL to upload the file to.
 * @param filePath The absolute path of the file to upload. The path is used as-is: a relative
 * path resolves against the process's current working directory, and it is not checked against
 * the file system plugin's scope.
 * @param progressHandler A callback invoked with upload progress updates. Progress is reported as
 * the file is read into the request body, so it can reach the total before the server has
 * received all the data and replied.
 * @param headers Additional request headers to send with the upload. The `Content-Length` header
 * is always set from the file size.
 * @param method The HTTP method used to send the file. Defaults to {@link HttpMethod.Post}.
 * @returns A promise resolving to the response body as text. It rejects if the file cannot be
 * read, if the request fails, or with `request failed with status code <code>: <body>` if the
 * server replies with a non-2xx status.
 *
 * @since 2.0.0
 */
async function upload(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  // TODO: V3 - Combine headers and methods into one `options` object
  headers?: Map<string, string>,
  method?: HttpMethod
): Promise<string> {
  const ids = new Uint32Array(1)
  window.crypto.getRandomValues(ids)
  const id = ids[0]

  const onProgress = new Channel<ProgressPayload>()
  if (progressHandler) {
    onProgress.onmessage = progressHandler
  }

  return await invoke('plugin:upload|upload', {
    id,
    url,
    filePath,
    headers: headers ?? {},
    method: method ?? HttpMethod.Post,
    onProgress
  })
}

/**
 * Downloads a file from a given URL and writes it to the given path on disk.
 *
 * @example
 * ```typescript
 * import { download } from '@tauri-apps/plugin-upload';
 * import { appDataDir, join } from '@tauri-apps/api/path';
 *
 * await download(
 *   'https://example.com/file-download-link',
 *   await join(await appDataDir(), 'file.txt'),
 *   ({ progressTotal, total }) => console.log(`Downloaded ${progressTotal} of ${total} bytes`),
 *   new Map([['Authorization', 'Bearer <token>']])
 * );
 * ```
 *
 * @param url The URL to download the file from.
 * @param filePath The absolute path to save the file to. It must include the file name, and its
 * parent directory must already exist. The path is used as-is: a relative path resolves against
 * the process's current working directory, and it is not checked against the file system
 * plugin's scope.
 * @param progressHandler A callback invoked with download progress updates. The reported
 * `total` will be `0` if the server did not send a `Content-Length` header or the response body
 * is compressed.
 * @param headers Additional request headers to send with the download request.
 * @param body An optional request body. When provided, the download is requested with an HTTP
 * `POST` request using this value as the body; otherwise an HTTP `GET` request is used.
 * @returns A promise that resolves once the whole response has been written to `filePath`. It
 * rejects if the request fails, if the file cannot be written, or with
 * `request failed with status code <code>: <body>` if the server replies with a non-2xx status.
 *
 * @since 2.0.0
 */
async function download(
  url: string,
  filePath: string,
  progressHandler?: ProgressHandler,
  headers?: Map<string, string>,
  body?: string
): Promise<void> {
  const ids = new Uint32Array(1)
  window.crypto.getRandomValues(ids)
  const id = ids[0]

  const onProgress = new Channel<ProgressPayload>()
  if (progressHandler) {
    onProgress.onmessage = progressHandler
  }

  await invoke('plugin:upload|download', {
    id,
    url,
    filePath,
    headers: headers ?? {},
    onProgress,
    body
  })
}

export { download, upload, HttpMethod }

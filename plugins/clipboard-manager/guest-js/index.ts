// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write to the system clipboard.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'
import { Image, transformImage } from '@tauri-apps/api/image'

/**
 * Writes plain text to the clipboard.
 * @example
 * ```typescript
 * import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';
 * await writeText('Tauri is awesome!');
 * assert(await readText(), 'Tauri is awesome!');
 * ```
 *
 * @param text The plain text to write to the clipboard.
 * @param opts Additional configuration for the write operation.
 * @param opts.label A label describing the copied content. **Android only**, ignored on other platforms.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function writeText(
  text: string,
  opts?: { label?: string }
): Promise<void> {
  await invoke('plugin:clipboard-manager|write_text', {
    label: opts?.label,
    text
  })
}

/**
 * Gets the clipboard content as plain text.
 * @example
 * ```typescript
 * import { readText } from '@tauri-apps/plugin-clipboard-manager';
 * const clipboardText = await readText();
 * ```
 * @returns A promise resolving to the clipboard contents as plain text.
 * @since 2.0.0
 */
async function readText(): Promise<string> {
  return await invoke('plugin:clipboard-manager|read_text')
}

/**
 * Writes an image to the clipboard.
 *
 * The image can be given as:
 * - an {@link Image}, passed to Rust by its resource id. Use `Image.new(rgba, width, height)`
 *   from `@tauri-apps/api/image` for raw RGBA pixels, or pass an image returned by {@link readImage};
 * - the bytes of an encoded PNG or ICO file (`Uint8Array`, `ArrayBuffer` or `number[]`);
 * - a string, which is the path to a PNG or ICO file.
 *
 * Encoded bytes and file paths are decoded by Tauri, which requires enabling the `image-png`
 * and/or `image-ico` feature of the `tauri` crate. Without them, only {@link Image} works.
 *
 * **Security:** file paths are opened without any file system scope check. Granting
 * `clipboard-manager:allow-write-image` lets the webview copy any PNG/ICO file the app can read to
 * the system clipboard (and read its pixels back if `clipboard-manager:allow-read-image` is granted too).
 *
 * #### Platform-specific
 *
 * - **Android / iOS:** Not supported.
 *
 * @example
 * ```typescript
 * import { Image } from '@tauri-apps/api/image';
 * import { writeImage } from '@tauri-apps/plugin-clipboard-manager';
 *
 * // a 2x1 image from raw RGBA pixels: a red pixel and a green pixel
 * const image = await Image.new([255, 0, 0, 255, 0, 255, 0, 255], 2, 1);
 * await writeImage(image);
 * await image.close();
 *
 * // the bytes of a PNG file (needs the `image-png` feature of the `tauri` crate)
 * const png = await (await fetch('/icon.png')).arrayBuffer();
 * await writeImage(png);
 * ```
 *
 * @param image The image to write: an {@link Image}, the bytes of an encoded PNG/ICO file, or the path to one.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function writeImage(
  image: string | Image | Uint8Array | ArrayBuffer | number[]
): Promise<void> {
  await invoke('plugin:clipboard-manager|write_image', {
    image: transformImage(image)
  })
}

/**
 * Gets the clipboard content as an image.
 *
 * The returned {@link Image} keeps the decoded RGBA pixels alive on the Rust side until it is
 * closed. Call `image.close()` once you are done with it, otherwise the memory is only freed
 * when the webview is destroyed.
 *
 * Rejects if the clipboard does not hold an image.
 *
 * #### Platform-specific
 *
 * - **Android / iOS:** Not supported.
 *
 * @example
 * ```typescript
 * import { readImage } from '@tauri-apps/plugin-clipboard-manager';
 *
 * const image = await readImage();
 * const { width, height } = await image.size();
 * const rgba = await image.rgba();
 * await image.close();
 *
 * // `rgba` holds raw pixels, not an encoded file: draw it on a canvas to display it
 * const canvas = document.createElement('canvas');
 * canvas.width = width;
 * canvas.height = height;
 * canvas
 *   .getContext('2d')
 *   ?.putImageData(new ImageData(new Uint8ClampedArray(rgba), width, height), 0, 0);
 * ```
 * @returns A promise resolving to the clipboard contents as an {@link Image}.
 * @since 2.0.0
 */
async function readImage(): Promise<Image> {
  return await invoke<number>('plugin:clipboard-manager|read_image').then(
    (rid) => new Image(rid)
  )
}

/**
 * Writes HTML to the clipboard, along with an optional plain text fallback.
 *
 * Applications that can paste HTML get the markup; the others, and {@link readText}, get `altText`.
 *
 * #### Platform-specific
 *
 * - **Android / iOS:** Not supported.
 *
 * @example
 * ```typescript
 * import { writeHtml, readText } from '@tauri-apps/plugin-clipboard-manager';
 * await writeHtml('<h1>Tauri is awesome!</h1>', 'Tauri is awesome!');
 * // reading the clipboard as text returns the plain text fallback, not the markup
 * assert(await readText(), 'Tauri is awesome!');
 * ```
 *
 * @param html The HTML markup to write to the clipboard.
 * @param altText The plain text fallback written alongside the HTML, used by targets that cannot render it.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function writeHtml(html: string, altText?: string): Promise<void> {
  await invoke('plugin:clipboard-manager|write_html', {
    html,
    altText
  })
}

/**
 * Clears the clipboard.
 *
 * #### Platform-specific
 *
 * - **Android:** Only supported on SDK 28+. For older releases we write an empty string to the clipboard instead.
 *
 * @example
 * ```typescript
 * import { clear } from '@tauri-apps/plugin-clipboard-manager';
 * await clear();
 * ```
 * @since 2.0.0
 */
async function clear(): Promise<void> {
  await invoke('plugin:clipboard-manager|clear')
}

export { writeText, readText, writeHtml, clear, readImage, writeImage }

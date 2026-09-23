// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import UIKit
import MobileCoreServices
import PhotosUI
import Photos
import Tauri

public class FilePickerController: NSObject {
  var plugin: DialogPlugin
    
	init(_ dialogPlugin: DialogPlugin) {
		plugin = dialogPlugin
	}

	private func dismissViewController(_ viewControllerToPresent: UIViewController, completion: (() -> Void)? = nil) {
		viewControllerToPresent.dismiss(animated: true, completion: completion)
	}

	/// ## In which cases do we need to save a copy of a file selected by a user to the app sandbox?
	/// In short, only when the file is **not** selected using UIDocumentPickerDelegate.
	/// For the rest of the cases, we need to write a copy of the file to the app sandbox. 
	/// 
	/// For PHPicker (used for photos and videos), `NSItemProvider.loadFileRepresentation` returns a temporary file URL that is deleted after the completion handler.
	/// The recommendation is to [Persist](https://developer.apple.com/documentation/foundation/nsitemprovider/2888338-loadfilerepresentation) the file by moving/copying
	/// it to your app’s directory within the completion handler.
	/// 
	/// If available, `loadInPlaceFileRepresentation` can open a file in place; Photos assets typically do not support true in-place access,
	/// so fall back to persisting a local file.
	/// Ref: https://developer.apple.com/documentation/foundation/nsitemprovider/2888335-loadinplacefilerepresentation
	/// 
	/// For UIDocumentPicker, prefer "open in place" and avoid copying when possible.
	/// Ref: https://developer.apple.com/documentation/uikit/uidocumentpickerviewcontroller
	private func saveTemporaryFile(_ sourceUrl: URL) throws -> URL {

		var directory = URL(fileURLWithPath: NSTemporaryDirectory())
		if let cachesDirectory = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first {
			directory = cachesDirectory
		}
		
		// Copy into a folder of its own: files picked at the same time or in earlier picks can share
		// a name (common for PHPicker representations), and must not overwrite or delete each other.
		let targetDirectory = directory.appendingPathComponent(UUID().uuidString, isDirectory: true)
		try FileManager.default.createDirectory(at: targetDirectory, withIntermediateDirectories: true)
		let targetUrl = targetDirectory.appendingPathComponent(sourceUrl.lastPathComponent)

		try FileManager.default.copyItem(at: sourceUrl, to: targetUrl)
		return targetUrl
	}
}

extension FilePickerController: UIDocumentPickerDelegate {
	public func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
		self.plugin.onFilePickerEvent(.selected(urls))
	}

	public func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
		self.plugin.onFilePickerEvent(.cancelled)
	}
}

extension FilePickerController: UIImagePickerControllerDelegate, UINavigationControllerDelegate, UIPopoverPresentationControllerDelegate {
	public func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
		dismissViewController(picker)
		self.plugin.onFilePickerEvent(.cancelled)
	}

	public func popoverPresentationControllerDidDismissPopover(_ popoverPresentationController: UIPopoverPresentationController) {
		self.plugin.onFilePickerEvent(.cancelled)
	}

	public func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
		self.plugin.onFilePickerEvent(.cancelled)
	}

	public func imagePickerController(_ picker: UIImagePickerController, didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]) {
		dismissViewController(picker) {
			if let url = info[.mediaURL] as? URL {
				do {
					let temporaryUrl = try self.saveTemporaryFile(url)
					self.plugin.onFilePickerEvent(.selected([temporaryUrl]))
				} catch {
					self.plugin.onFilePickerEvent(.error("Failed to create a temporary copy of the file"))
				}
			} else {
				self.plugin.onFilePickerEvent(.cancelled)
			}
		}
	}
}

@available(iOS 14, *)
extension FilePickerController: PHPickerViewControllerDelegate {
	public func picker(_ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]) {
		dismissViewController(picker)
		if results.first == nil {
			self.plugin.onFilePickerEvent(.cancelled)
			return
		}
		var temporaryUrls: [URL] = []
		var errorMessage: String?
		let dispatchGroup = DispatchGroup()
		for result in results {
			if errorMessage != nil {
				break
			}
			if result.itemProvider.hasItemConformingToTypeIdentifier(UTType.movie.identifier) {
				dispatchGroup.enter()
				result.itemProvider.loadFileRepresentation(forTypeIdentifier: UTType.movie.identifier, completionHandler: { url, error in
					defer {
						dispatchGroup.leave()
					}
					if let error = error {
						errorMessage = error.localizedDescription
						return
					}
					guard let url = url else {
						errorMessage = "Unknown error"
						return
					}
					do {
						// We have to make a copy of the file to the app sandbox here, as PHPicker returns an NSItemProvider with either an ephemeral file URL or content that is deleted after the completion handler.
						// This is a different behavior from UIDocumentPicker, where the file can either be copied to the app sandbox or opened in place, and then accessed with `startAccessingSecurityScopedResource`.
						let temporaryUrl = try self.saveTemporaryFile(url)
						temporaryUrls.append(temporaryUrl)
					} catch {
						errorMessage = "Failed to create a temporary copy of the file"
					}
				})
			} else if result.itemProvider.hasItemConformingToTypeIdentifier(UTType.image.identifier) {
				dispatchGroup.enter()
				result.itemProvider.loadFileRepresentation(forTypeIdentifier: UTType.image.identifier, completionHandler: { url, error in
					defer {
						dispatchGroup.leave()
					}
					if let error = error {
						errorMessage = error.localizedDescription
						return
					}
					guard let url = url else {
						errorMessage = "Unknown error"
						return
					}
					do {
						// We have to make a copy of the file to the app sandbox here, as PHPicker returns an NSItemProvider with either an ephemeral file URL or content that is deleted after the completion handler.
						// This is a different behavior from UIDocumentPicker, where the file can either be copied to the app sandbox or opened in place, and then accessed with `startAccessingSecurityScopedResource`.
						let temporaryUrl = try self.saveTemporaryFile(url)
						temporaryUrls.append(temporaryUrl)
					} catch {
						errorMessage = "Failed to create a temporary copy of the file"
					}
				})
			} else {
				errorMessage = "Unsupported file type identifier"
			}
		}
		dispatchGroup.notify(queue: .main) {
			if let errorMessage = errorMessage {
				self.plugin.onFilePickerEvent(.error(errorMessage))
				return
			}
			self.plugin.onFilePickerEvent(.selected(temporaryUrls))
		}
	}
}
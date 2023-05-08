/* 
    Storage solution for uploading images and storing them using
    UUID's and then handling adding them to a form for uploading
*/

import type { ImageRef } from "$lib/socket/models";
import { v4 as uuidv4 } from "uuid";
import imageCompression from "browser-image-compression";
import { writable, type Writable } from "svelte/store";

export interface StoredImage {
  /// The file UUID
  uuid: ImageRef;

  /// The name of the file
  name: string;

  /// The size of the file
  size: number;

  /// The actual file
  blob: Blob;
}

type PreviewStore = Record<ImageRef, string | undefined>;

// Store mapping image references to their loaded image preview
export const imagePreviewStore: Writable<PreviewStore> = writable({});
// Store for storing the loaded images
export const imageStore: Writable<StoredImage[]> = writable([]);
// Store for state to manage selecting an image
export const selectImageStore: Writable<UploadImageCallback | null> =
  writable(null);

type UploadImageCallback = (image: StoredImage | null) => void;

/**
 * Creates a promise that resolves once an
 * image has been selected
 *
 * @returns
 */
export function selectImage(): Promise<StoredImage | null> {
  return new Promise((resolve) => {
    selectImageStore.set(resolve);
  });
}

/**
 * Handles clearing the select image callback state
 * rejecting the callback
 */
export function clearSelectImage() {
  selectImageStore.update((callback) => {
    // Reject the callback because nothing was sent back
    if (callback !== null) {
      callback(null);
    }

    return null;
  });
}

/**
 * Handles responding to a select image callback
 * with the selected image. Resets the callback to
 * null
 *
 * @param image The selected image
 */
export function consumeSelectImage(image: StoredImage) {
  selectImageStore.update((callback) => {
    // Call the callback with the image
    if (callback !== null) {
      callback(image);
    }

    return null;
  });
}

export async function uploadFile(
  file: File,
  onProgress: (progress: number) => void
): Promise<void> {
  const uuid: string = uuidv4();

  // Compress the image blob
  const compressed: File = await imageCompression(file, {
    // Try to make the image as small as possible
    maxSizeMB: 0.8,
    // Send the compression progress to the callback
    onProgress
  });

  // Update the image store with the new image
  imageStore.update((store) => {
    // Update the data store
    store.push({
      uuid,
      name: file.name,
      size: file.size,
      blob: compressed
    });

    return store;
  });

  // Load the image preview in the background
  loadImagePreview(compressed, uuid);
}

/**
 * Loads the preview image data URL from the provided image
 * blob storing it in the image preview store using the
 * provided UUID
 *
 * @param image The image to load the preview for
 * @param uuid  The UUID for storing the preview image
 * @returns     The promise to once the image is stored
 */
export async function loadImagePreview(
  image: Blob,
  uuid: string
): Promise<void> {
  const reader = new FileReader();

  // Await reading finished
  await new Promise((resolve, reject) => {
    // Handle the different events
    reader.onload = resolve;
    reader.onerror = reject;
    reader.onabort = reject;

    // Read the image as a data URL
    reader.readAsDataURL(image);
  });

  const previewUrl = reader.result as string;

  // Update the store to include the preview URL
  imagePreviewStore.update((store) => {
    store[uuid] = previewUrl;
    return store;
  });
}

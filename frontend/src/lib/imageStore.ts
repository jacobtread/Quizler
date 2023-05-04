
/* 
    Storage solution for uploading images and storing them using
    UUID's and then handling adding them to a form for uploading
*/

import type { ImageRef } from "./socket/models";
import { v4 as uuidv4 } from 'uuid';
import imageCompression from "browser-image-compression"
import { writable, type Writable } from "svelte/store";


interface StoredImage {
    /// The file UUID
    uuid: ImageRef;

    /// The name of the file
    name: string;

    /// The size of the file
    size: number;

    /// The actual file
    blob: Blob,

    /// Data url for displaying the file
    previewUrl: string | null;
}

export const imageStore: Writable<StoredImage[]> = writable([]);



export function uploadFile(file: File, onProgress: (progress: number) => void): Promise<void> {
    const uuid: string = uuidv4();

    // Future for compressing the image 
    const compressPromise: Promise<Blob> = imageCompression(file, {
        // Try to make the image as small as possible
        maxSizeMB: 0.8,
        // Send the compression progress to the callback
        onProgress
    })
        // Update store with the compressed file
        .then((compressed) => {

            imageStore.update((store) => {
                // Update the data store
                store.push({
                    uuid,
                    name: file.name,
                    size: file.size,
                    blob: compressed,
                    previewUrl: null
                });

                return store;
            });

            return compressed;
        });

    return compressPromise
        .then((compressed) => new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = (event) => {
                const previewUrl = event.target.result as string
                imageStore.update((store) => {
                    let image = store.find((value) => value.uuid == uuid);

                    if (image) image.previewUrl = previewUrl;
                    resolve();
                    return store;
                });
            };
            reader.onerror = reject;
            reader.onabort = reject;
            reader.readAsDataURL(compressed);
        }));
}

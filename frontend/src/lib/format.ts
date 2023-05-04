/* 
    Reading and writing logic for the quizler file format
*/

import { get } from "svelte/store";
import { imageStore, loadImagePreview, type StoredImage } from "./imageStore";
import type { ImageRef, Question, TimingConfig, UploadConfig } from "./socket/models";

export interface StoredImageMap {
    /// The file UUID
    uuid: string;

    /// The name of the file
    name: string;

    /// The size of the file
    size: number;

    /// The actual file
    data: number[],
}

async function mapImage(image: StoredImage): Promise<StoredImageMap> {
    const buffer = await image.blob.arrayBuffer();
    return {
        uuid: image.uuid,
        name: image.name,
        size: image.size,
        data: Array.from(new Uint8Array(buffer)) as number[]
    }
}

export function loadQuizFile(file: Blob): Promise<UploadConfig> {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = (event) => {
            const data = event.target.result as string
            const out = loadQuiz(data);
            resolve(out);
        };
        reader.onerror = reject;
        reader.onabort = reject;
        reader.readAsText(file);
    })
}

function loadQuiz(data: string): UploadConfig {
    imageStore.set([]);

    let obj = JSON.parse(data) as SerializedQuiz;

    for (const mapped of obj.images) {

        const b = new Uint8Array(mapped.data);
        const b2 = new Blob([b.buffer]);

        imageStore.update((store) => {
            // Update the data store
            store.push({
                uuid: mapped.uuid,
                name: mapped.name,
                size: mapped.size,
                blob: b2,
                previewUrl: null
            });

            return store;
        });

        loadImagePreview(b2, mapped.uuid);
    }

    return obj.config;
}

interface SerializedQuiz {
    config: UploadConfig,
    images: StoredImageMap[]
}

export async function saveQuiz(
    config: UploadConfig,
) {
    const images = get(imageStore);

    let promises: Promise<StoredImageMap>[] = [];
    for (const image of images) {
        promises.push(mapImage(image))
    };

    let mappedImages = await Promise.all(promises);

    let output = {
        config,
        images: mappedImages
    }

    let outputJSON = JSON.stringify(output);

    const URL = window.webkitURL ?? window.URL;
    const id = 'tmpDownload'
    let element: HTMLAnchorElement = document.getElementById(id) as (HTMLAnchorElement | null) ?? ((): HTMLAnchorElement => {
        const element = document.createElement('a') as HTMLAnchorElement
        element.id = id
        return element
    })()
    const safeName: string = config.basic.name.replace(/[ ^\/]/g, '_')
    const blob = new Blob([outputJSON], { 'type': 'application/json' });
    element.download = safeName + '.quizler';
    element.href = URL.createObjectURL(blob);
    element.dataset.downloadurl = ['application/json', element.download, element.href].join(':');
    element.style.display = 'none';
    element.click()
}
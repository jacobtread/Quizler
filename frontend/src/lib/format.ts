// Reading and writing logic for the quizler file format

import { get } from "svelte/store";
import { imageStore, loadImagePreview, type StoredImage } from "./imageStore";
import type { Question, TimingConfig, UploadConfig } from "./socket/models";

interface SerializedQuiz {
  name: string;
  text: string;
  timing: TimingConfig;
  questions: Question[];
  images: SerializedImage[];
}

export interface DeserializedQuiz {
  name: string;
  text: string;
  timing: TimingConfig;
  questions: Question[];
}

interface SerializedImage {
  uuid: string;
  name: string;
  size: number;
  data: number[];
}

/**
 * Converts the provided stored image into a JSON
 * serializable version by converting the stored
 * blob into a array of bytes
 *
 * @param image The image to convert
 * @returns The converted image
 */
async function serializeImage(image: StoredImage): Promise<SerializedImage> {
  const buffer = await image.blob.arrayBuffer();
  return {
    uuid: image.uuid,
    name: image.name,
    size: image.size,
    data: Array.from(new Uint8Array(buffer)) as number[]
  };
}

/**
 * Serializes the provided quiz contents and starts a
 * download for the file
 *
 * @param name      The quiz name (Also the file name)
 * @param text      The quiz text
 * @param timing    The quiz timing
 * @param questions The quiz questions
 */
export async function saveQuiz(
  name: string,
  text: string,
  timing: TimingConfig,
  questions: Question[]
) {
  // Convert the stored images into a serializable form
  const images: SerializedImage[] = await Promise.all(
    get(imageStore).map(serializeImage)
  );

  // Create the object to serialize as the quiz file
  const output: SerializedQuiz = { name, text, timing, questions, images };

  // Save the quiz file
  saveObject(name, "quizler", output);
}

/**
 * Starts a file download for a file containing the
 * JSON string version of the provided object
 *
 * @param name   The output file name
 * @param ext    The output file extension
 * @param object The object to stringify
 */
function saveObject(name: string, ext: string, object: any) {
  const json = JSON.stringify(object);
  const safeName: string = name.replace(/[ ^\/]/g, "_");
  const blob = new Blob([json], { type: "application/json" });

  const URL = window.webkitURL ?? window.URL;
  const ID = "tmpDownload";

  let element: HTMLAnchorElement | null = document.getElementById(
    ID
  ) as HTMLAnchorElement | null;
  if (element == null) {
    element = document.createElement("a");
    element.id = ID;
  }

  element.download = safeName + "." + ext;
  element.href = URL.createObjectURL(blob);
  element.dataset.downloadurl = [
    "application/json",
    element.download,
    element.href
  ].join(":");
  element.style.display = "none";
  element.click();

  document.removeChild(element);
}

/**
 * Loads a quiz from the provided file blob
 *
 * @param file The file to load the quiz from
 * @returns The loaded quiz data
 */
export async function loadQuiz(file: Blob): Promise<DeserializedQuiz> {
  // Load the string contents of the file blob
  const data: string = await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (event) => {
      const data = event.target.result as string;
      resolve(data);
    };
    reader.onerror = reject;
    reader.onabort = reject;
    reader.readAsText(file);
  });

  // TODO: Actually validate that this input is correct
  const obj = JSON.parse(data) as SerializedQuiz;

  for (const { data, uuid, name, size } of obj.images) {
    // Convert the input data array into a blob
    const dataArray = new Uint8Array(data);
    const blob = new Blob([dataArray.buffer]);

    // Add the image to the image store
    imageStore.update((store) => {
      // Update the data store
      store.push({ uuid, name, size, blob, previewUrl: null });

      return store;
    });

    // Trigger the image preview loading
    loadImagePreview(blob, uuid);
  }

  return {
    name: obj.name,
    text: obj.text,
    questions: obj.questions,
    timing: obj.timing
  };
}

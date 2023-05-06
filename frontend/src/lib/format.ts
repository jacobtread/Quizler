// Reading, writing and validation logic for the quizler file format

import { get } from "svelte/store";
import {
  imageStore,
  loadImagePreview,
  type StoredImage
} from "$stores/imageStore";
import {
  type Question,
  type TimingConfig,
  questionSchema
} from "$lib/socket/models";
import { z } from "zod";

/// Schema used for parsing and validating the file format
const fileFormatSchema = z.object({
  name: z.string(),
  text: z.string(),
  timing: z.object({
    wait_time: z.number()
  }),
  questions: z.array(questionSchema).min(1),
  images: z.array(
    z.object({
      uuid: z.string().uuid(),
      name: z.string(),
      size: z.number(),
      type: z.string(),
      data: z.array(z.number())
    })
  )
});

type QuizFormat = z.infer<typeof fileFormatSchema>;
type SerializedImage = QuizFormat["images"][0];
type LoadedQuiz = Omit<QuizFormat, "images">;

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
    type: image.blob.type,
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
  const output: QuizFormat = { name, text, timing, questions, images };

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
function saveObject<T>(name: string, ext: string, object: T) {
  const json = JSON.stringify(object);
  const safeName: string = name.replace(/[ ^/]/g, "_");
  const blob = new Blob([json], { type: "application/json" });

  const URL = window.webkitURL ?? window.URL;
  const element: HTMLAnchorElement = document.createElement("a");

  element.download = safeName + "." + ext;
  element.href = URL.createObjectURL(blob);
  element.dataset.downloadurl = [
    "application/json",
    element.download,
    element.href
  ].join(":");
  element.style.display = "none";
  document.body.appendChild(element);
  element.click();
  document.body.removeChild(element);
}

/**
 * Loads a quiz from the provided file blob
 *
 * @param file The file to load the quiz from
 * @throws {ZodError} If the validation failed
 * @returns The loaded quiz data
 */
export async function loadQuiz(file: Blob): Promise<LoadedQuiz> {
  const reader = new FileReader();

  // Await the reading process
  await new Promise((resolve, reject) => {
    reader.onload = resolve;
    reader.onerror = reject;
    reader.onabort = reject;

    // Load the file as a string
    reader.readAsText(file);
  });

  const data = reader.result as string;

  const parsed = JSON.parse(data);
  const obj: QuizFormat = fileFormatSchema.parse(parsed);

  for (const { data, uuid, name, type, size } of obj.images) {
    // Convert the input data array into a blob
    const dataArray = new Uint8Array(data);
    const blob = new Blob([dataArray.buffer], { type });

    // Add the image to the image store
    imageStore.update((store) => {
      // Update the data store
      store.push({ uuid, name, size, blob });

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

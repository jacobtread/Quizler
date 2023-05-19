// Reading, writing and validation logic for the quizler file format

import {
  imageStore,
  loadImagePreview,
  type StoredImage
} from "$stores/imageStore";
import type { CreateData } from "$stores/createStore";
import { questionSchema, NameFiltering } from "$api/models";
import { z } from "zod";

// Schema used for parsing and validating the file format
const fileFormatSchema = z.object({
  name: z.string(),
  text: z.string(),
  max_players: z.number(),
  filtering: z.string(),
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
 * Serializes the provided quiz contents into a blob
 *
 * @param data The quiz data
 * @param storeImages The stored images
 * @returns Promise to the encoded quiz blob
 */
export async function createQuizBlob(
  data: CreateData,
  storeImages: StoredImage[]
): Promise<Blob> {
  // Convert the stored images into a serializable form
  const images: SerializedImage[] = await Promise.all(
    storeImages.map(serializeImage)
  );

  // Create the object to serialize as the quiz file
  const output: QuizFormat = {
    ...data,
    images
  };

  const json = JSON.stringify(output);

  // Blob created from the serialized JSON
  return new Blob([json], { type: "application/json" });
}

/**
 * Loads a quiz from the provided file blob
 *
 * @param file The file to load the quiz from
 * @throws {ZodError} If the validation failed
 * @returns The loaded quiz data
 */
export async function loadQuizBlob(file: Blob): Promise<CreateData> {
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

      // Remove duplicates from loading the file again
      return store.filter((value) => value.uuid !== uuid);
    });

    // Trigger the image preview loading
    loadImagePreview(blob, uuid);
  }

  return {
    name: obj.name,
    text: obj.text,
    max_players: obj.max_players,
    filtering: obj.filtering as NameFiltering,
    questions: obj.questions,
    timing: obj.timing
  };
}

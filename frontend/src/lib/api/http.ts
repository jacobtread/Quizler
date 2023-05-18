import type { CreateData } from "$lib/stores/createStore";
import type { StoredImage } from "$lib/stores/imageStore";
import type { CreatedResponse, Question } from "./models";

/**
 * Obtains a server url for the provided path.
 *
 * (e.g. path = "/test" output = http://localhost/test)
 *
 * @param path The route path
 * @returns The created URL
 */
export function getServerURL(path: string): URL {
  return new URL(
    path,
    // Use localhost for dev environments otherwise extract from the origin
    import.meta.env.DEV ? "http://localhost" : window.location.origin
  );
}

/**
 * Creates a quiz image URL from the provided game
 * token and UUID to the image
 *
 * @param token The game token
 * @param uuid  The image UUID
 * @returns     The URL to the image
 */
export function formatImageUrl(token: string, uuid: string): string {
  return getServerURL(`/api/quiz/${token}/${uuid}`).toString();
}

/**
 * Calls the quiz creation endpoint uploading the quiz config
 * and all the referenced images returns a Promise to the UUID
 * of the created game or an Error
 *
 * @param config The quiz config to use
 * @param images The images from the image store
 * @param progress Progress callback for the upload progress
 * @returns Promise to the UUID of the created game
 */
export function createHttp(
  config: CreateData,
  images: StoredImage[],
  progress: (event: ProgressEvent) => void
): Promise<string> {
  return new Promise((resolve, reject) => {
    // Create the form to upload
    const form: FormData = createUploadForm(config, images);

    const request: XMLHttpRequest = new XMLHttpRequest();

    request.upload.onprogress = progress;
    request.responseType = "json";

    // Handle success
    request.onload = () => {
      const status: number = Math.floor(request.status / 100);

      // Handle non 2xx status codes
      if (status !== 2) {
        console.error("Failed invalid request", request);
        return reject(
          new Error(
            "Invalid request try reloading the page or updating Quizler"
          )
        );
      }

      // Okay responses
      const response: CreatedResponse = request.response;
      resolve(response.uuid);
    };

    // Handle all failure callbacks
    request.onerror =
      request.ontimeout =
      request.onabort =
        () => reject(new Error("Failed to connect"));

    // Create the URL to the create endpoint
    const url = getServerURL("/api/quiz");

    // Set the request method and URL
    request.open("POST", url);

    // Send the multipart form body
    request.send(form);
  });
}

/**
 * Creates the upload form data for the provided quiz
 * config and images. Filters the images so that only
 * those referenced by questions are included
 *
 * @param config The quiz config to use
 * @param images The images from the image store
 * @returns The created form data
 */
function createUploadForm(config: CreateData, images: StoredImage[]): FormData {
  // Create the form to upload
  const form = new FormData();
  // Append the config
  form.append("config", JSON.stringify(config));

  // Append the images to the form
  for (const image of images) {
    // Images require atleast 1 reference to be included
    const usage: Question | undefined = config.questions.find(
      (question) => question.image === image.uuid
    );
    if (usage === undefined) continue;

    form.append(image.uuid, image.blob);
  }

  return form;
}

/**
 * Attempts to preload the image for the provided
 * question using the game token
 *
 * Will attempt 5 times before failing and will
 * continue to ready state regardless of failure
 *
 * @param token    The question game token
 * @param question The question itself
 * @returns        Promise to the preloading complete
 */
export async function preloadImage(token: string, question: Question) {
  const imageRef = question.image;

  // Question didn't have any images to load
  if (imageRef === null) return;

  const MAX_ATTEMPTS = 6;

  let attempts: number = 0;

  const url: string = formatImageUrl(token, imageRef);

  while (attempts < MAX_ATTEMPTS) {
    try {
      // Attempt to load the image
      await new Promise((resolve, reject) => {
        const img = new Image();
        img.src = url;
        img.onload = resolve;
        img.onerror = reject;
      });

      console.debug("Preloaded question image", url);
      break;
    } catch (e) {
      console.error("Failed to preload image trying again", url, e);
      attempts += 1;
    }
  }
}

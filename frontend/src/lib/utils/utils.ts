import type { Question, TimerState } from "$lib/socket/models";

/**
 * Formats the provided timer as seconds
 *
 * @param timer The timer to format
 * @returns     The formatted time
 */
export function formatTime(timer: TimerState): string {
  const timeMs: number = timer.total - timer.elapsed;
  const seconds = timeMs / 1000;
  return seconds.toFixed(0);
}

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
 * Produces a random number between the provided
 * min and max (inclusive)
 *
 * @param min The minimum number
 * @param max The maxmimum number
 * @returns The random number
 */
export function randomRange(min: number, max: number): number {
  const value = Math.round(Math.random() * (max - min) + min);
  if (value > max) return max;
  if (value < min) return min;
  return value;
}

/**
 * Creates a deep copy of the provided value
 *
 * @param value The value to copy
 * @returns The copied value
 */
export function deepCopy<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

/**
 * Appends the Ordinal string to the end of the provided
 * number used for lists (e.g, 1st, 2nd, 3rd)
 *
 * @param n The number
 * @returns The number with the ordinal
 */
export function getNumberWithOrdinal(n: number): string {
  const s: string[] = ["th", "st", "nd", "rd"];
  const v: number = n % 100;
  return n + (s[(v - 20) % 10] || s[v] || s[0]);
}

/**
 * Attempts to put the browser into fullscren
 * mode for a better viewing experience
 */
export function tryFullscreen() {
  const documentElement = document.documentElement;
  if (documentElement.requestFullscreen === undefined) return false;

  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen({
      navigationUI: "hide"
    });
  }

  return tryFullscreen;
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

  /// Question didn't have any images to load
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

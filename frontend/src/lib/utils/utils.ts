import type { TimerState } from "$api/models";

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

export function shuffleArray<T>(array: T[]): T[] {
  const shuffleCount = randomRange(1, array.length);
  let changes = 0;
  while (changes < shuffleCount) {
    const first = randomRange(0, array.length - 1);
    const second = randomRange(0, array.length - 1);
    if (first !== second) {
      arraySwap(array, first, second);
      changes++;
    }
  }
  return array;
}

export function arraySwap<T>(array: T[], aIndex: number, bIndex: number): T[] {
  const a: T | undefined = array[aIndex];
  const b: T | undefined = array[bIndex];

  if (a !== undefined || b !== undefined) {
    // Swap the questions
    array[aIndex] = b;
    array[bIndex] = a;
  }

  return array;
}

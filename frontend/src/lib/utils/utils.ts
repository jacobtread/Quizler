/**
 * Formats the provided timer as seconds
 *
 * @param timeMs The time in milliseconds to format
 * @returns     The formatted time
 */
export function formatTime(timeMs: number): string {
  const seconds = timeMs / 1000;
  return seconds.toFixed(0);
}

/**
 * Formats the provided size in bytes in
 * bytes, kb, and mb units
 *
 * @param bytes The bytes to format
 */
export function formatBytes(bytes: number): string {
  const sizes: string[] = ["bytes", "kb", "mb"];
  if (bytes == 0) return "0 Byte";
  const i: number = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + sizes[i];
}

/**
 * Produces a random number between the provided
 * min and max (inclusive)
 *
 * @param min The minimum number
 * @param max The maximum number
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
 * Randomly shuffles the array. Ensures that at least length/2
 * changes are made to the array
 *
 * @param array The array to shuffle
 * @returns The reference to the provided array
 */
export function shuffleArray<T>(array: T[]): T[] {
  const shuffleCount = randomRange(1, array.length / 2);
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

/**
 * Swaps the two elements in the array at the provided
 * indexes if both indexes exist
 *
 * @param array The array to swap within
 * @param aIndex The first index
 * @param bIndex The second index
 * @returns The reference to the provided array
 */
export function arraySwap<T>(array: T[], aIndex: number, bIndex: number): T[] {
  const a: T | undefined = array[aIndex];
  const b: T | undefined = array[bIndex];

  // Ensure both elements exist
  if (a !== undefined || b !== undefined) {
    // Swap the elements
    array[aIndex] = b;
    array[bIndex] = a;
  }

  return array;
}

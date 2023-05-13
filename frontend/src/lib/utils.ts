import { DEBUG } from "$lib/constants";
import type { TimerState } from "$lib/socket/models";

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
 * Creates a quiz image URL from the provided game
 * token and UUID to the image
 *
 * @param token The game token
 * @param uuid  The image UUID
 * @returns     The URL to the image
 */
export function formatImageUrl(token: string, uuid: string): string {
  return new URL(
    `/api/quiz/${token}/${uuid}`,
    DEBUG ? "http://localhost" : window.location.origin
  ).toString();
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

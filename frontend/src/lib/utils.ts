import { DEBUG } from "./constants";
import type { TimerState } from "./socket/models";

export function formatTime(timer: TimerState): string {
  const timeMs: number = timer.total - timer.elapsed;
  const seconds = timeMs / 1000;
  return seconds.toFixed(0);
}

export function formatImageUrl(token: string, uuid: string): URL {
  return new URL(
    `/api/quiz/${token}/${uuid}`,
    DEBUG ? "http://localhost" : window.location.origin
  );
}

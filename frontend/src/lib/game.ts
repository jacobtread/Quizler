import { writable } from "svelte/store";
import { GameState, type GameConfig } from "./socket/models";

interface GameData {
  id: number;
  token: string;
  config: GameConfig;
}

export const gameState = writable<GameState>(GameState.Lobby);
export const gameData = writable<GameData | null>(null);

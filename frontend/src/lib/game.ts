import { writable } from "svelte/store";
import { GameState, type GameConfig, type SessionId } from "./socket/models";
import { AppState, appState } from "./state";
import { gameHost } from "./socket";

interface GameData {
  id: SessionId;
  token: string;
  config: GameConfig;
}

interface OtherPlayer {
  id: SessionId;
  name: string;
}

export const gameState = writable<GameState>(GameState.Lobby);
export const gameData = writable<GameData | null>(null);
export const players = writable<OtherPlayer[]>([]);

export function setJoinedGame(game: GameData, host: boolean) {
  gameHost.set(host);
  gameData.set(game);
  appState.set(AppState.Game);
}

export function clearGame() {
  gameState.set(GameState.Lobby);
  gameData.set(null);
  players.set([]);
}

export function setOtherPlayer(id: SessionId, name: string) {
  players.update((players) => {
    players.push({ id, name });
    return players;
  });
}

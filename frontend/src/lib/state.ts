import { writable } from "svelte/store";
import type { GameConfig, SessionId } from "./socket/models";

export const enum AppStateType {
  /// Home screen
  Home = 0,
  /// Join Game Screen
  Connect,
  /// Join game name selection screen
  Join,
  /// Create game screen
  Create,
  /// App is in a game
  Game
}

export interface GameData {
  id: SessionId;
  token: string;
  config: GameConfig;
  host: boolean;
}

export type AppState =
  | { ty: AppStateType.Home }
  | { ty: AppStateType.Connect }
  | { ty: AppStateType.Join; token: string }
  | { ty: AppStateType.Create }
  | { ty: AppStateType.Game; gameData: GameData };

export const appState = writable<AppState>({
  ty: AppStateType.Home
});

export function setHome() {
  appState.set({ ty: AppStateType.Home });
}

export function setCreate() {
  appState.set({ ty: AppStateType.Create });
}

export function setConnect() {
  appState.set({ ty: AppStateType.Connect });
}

export function setJoin(token: string) {
  appState.set({ ty: AppStateType.Join, token });
}

export function setGame(gameData: GameData) {
  appState.set({ ty: AppStateType.Game, gameData });
}

import { writable } from "svelte/store";
import type { GameConfig, GameToken, Question, SessionId } from "$api/models";

export const enum AppStateType {
  /// Home screen
  Home = 0,
  // Join Game Screen
  Connect,
  // Create game screen
  Create,
  // Editing question screen
  Editing,
  // App is in a game
  Game
}

export interface GameData {
  // ID of the current player
  id: SessionId;
  // Current game token
  token: GameToken;
  // Current game config
  config: GameConfig;
  // Whether we are the host
  host: boolean;
  // The current player name
  name?: string | undefined;
}

export type AppState =
  | { ty: AppStateType.Home }
  | { ty: AppStateType.Connect }
  | { ty: AppStateType.Create }
  | { ty: AppStateType.Editing; question: Question }
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

export function setGame(gameData: GameData) {
  appState.set({ ty: AppStateType.Game, gameData });
}

export function setEditing(question: Question) {
  appState.set({ ty: AppStateType.Editing, question });
}

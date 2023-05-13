import { writable } from "svelte/store";
import type { GameConfig, Question, SessionId } from "$lib/socket/models";
import { deepCopy } from "$lib/utils";

export const enum AppStateType {
  /// Home screen
  Home = 0,
  /// Join Game Screen
  Connect,
  /// Join game name selection screen
  Join,
  /// Create game screen
  Create,
  /// Editing question screen
  Editing,
  /// App is in a game
  Game
}

export interface GameData {
  // ID of the current player
  id: SessionId;
  // Current game token
  token: string;
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
  | { ty: AppStateType.Join; token: string }
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

export function setJoin(token: string) {
  appState.set({ ty: AppStateType.Join, token });
}

export function setGame(gameData: GameData) {
  appState.set({ ty: AppStateType.Game, gameData });
}

export function setEditing(question: Question) {
  question = deepCopy(question);
  appState.set({ ty: AppStateType.Editing, question });
}

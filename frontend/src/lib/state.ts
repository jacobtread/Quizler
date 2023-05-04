import { writable } from "svelte/store";

export const enum AppState {
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

export const appState = writable<AppState>(AppState.Home);

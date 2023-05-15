import Connect from "$lib/pages/Connect.svelte";
import Create from "$lib/pages/Create.svelte";
import Game from "$lib/pages/Game.svelte";
import Home from "$lib/pages/Home.svelte";
import Join from "$lib/pages/Join.svelte";
import type { ComponentProps, ComponentType } from "svelte";
import { writable, type Writable } from "svelte/store";
import QuestionEditor from "./pages/QuestionEditor.svelte";
import type { GameConfig, Question, SessionId } from "./socket/models";
import { deepCopy } from "./utils/utils";

export const enum Route {
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

const routes = {
  [Route.Home]: Home,
  [Route.Create]: Create,
  [Route.Connect]: Connect,
  [Route.Join]: Join,
  [Route.Editing]: QuestionEditor,
  [Route.Game]: Game
};

type Routes = typeof routes;
type RouteKey = keyof Routes;

// Maps the provided input type to its actual component type
type MapToComponent<Input> = Input extends ComponentType<infer Component>
  ? Component
  : never;

type PropsOf<Key extends RouteKey> = ComponentProps<
  MapToComponent<Routes[Key]>
>;

type RouteState<C extends ComponentType, P> = {
  component: C;
  props: P;
};

export const routeState: Writable<RouteState<ComponentType, object>> = writable(
  {
    component: routes[Route.Home],
    props: {}
  }
);

export function setState<T extends RouteKey, V = PropsOf<T>>(key: T, value: V) {
  routeState.set({
    component: routes[key],
    props: value as object
  });

  return;
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

export function setHome() {
  setState(Route.Home, {});
}

export function setCreate() {
  setState(Route.Create, {});
}

export function setConnect() {
  setState(Route.Connect, {});
}

export function setJoin(token: string) {
  setState(Route.Join, { token });
}

export function setGame(gameData: GameData) {
  setState(Route.Game, { gameData });
}

export function setEditing(question: Question) {
  question = deepCopy(question);
  setState(Route.Editing, { question });
}

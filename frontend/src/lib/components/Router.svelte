<script context="module" lang="ts">
  import Connect from "$lib/pages/Connect.svelte";
  import Create from "$lib/pages/Create.svelte";
  import Game, { type GameData } from "$lib/pages/Game.svelte";
  import Home from "$lib/pages/Home.svelte";
  import Join from "$lib/pages/Join.svelte";
  import { onMount, type ComponentProps, type ComponentType } from "svelte";
  import { writable, type Writable } from "svelte/store";
  import QuestionEditor from "$pages/QuestionEditor.svelte";
  import type { Question } from "$lib/socket/models";
  import { deepCopy } from "$lib/utils/utils";

  const routes = {
    Home: Home,
    Create: Create,
    Connect: Connect,
    Join: Join,
    Editing: QuestionEditor,
    Game: Game
  };

  type Routes = typeof routes;
  type RouteKey = keyof Routes;

  // Maps the provided input type to its actual component type
  type MapToComponent<Input> = Input extends ComponentType<infer Component>
    ? Component
    : never;

  type PropsOf<Input> = ComponentProps<MapToComponent<Input>>;

  type RouteState<C extends ComponentType, P> = {
    component: C;
    props: P;
  };

  export const routeState: Writable<RouteState<ComponentType, object>> =
    writable({
      component: routes.Home,
      props: {}
    });

  export function setState<T extends RouteKey, V = PropsOf<Routes[T]>>(
    key: T,
    value: V
  ) {
    routeState.set({
      component: routes[key],
      props: value as object
    });

    return;
  }

  export function setHome() {
    setState("Home", {});
  }

  export function setCreate() {
    setState("Create", {});
  }

  export function setConnect() {
    setState("Connect", {});
  }

  export function setJoin(token: string) {
    setState("Join", { token });
  }

  export function setGame(gameData: GameData) {
    setState("Game", { gameData });
  }

  export function setEditing(question: Question) {
    question = deepCopy(question);
    setState("Editing", { question });
  }
</script>

<svelte:component this={$routeState.component} {...$routeState.props} />

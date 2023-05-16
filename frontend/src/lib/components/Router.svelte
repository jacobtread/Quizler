<script context="module" lang="ts">
  import type { ComponentProps, ComponentType } from "svelte";
  import { writable, type Writable } from "svelte/store";

  import Game from "$pages/Game.svelte";
  import QuestionEditor from "$pages/QuestionEditor.svelte";
  import Connect from "$pages/Connect.svelte";
  import Create from "$pages/Create.svelte";
  import Home from "$pages/Home.svelte";
  import Join from "$pages/Join.svelte";

  // Route definitions
  const routes = {
    Home: Home,
    Create: Create,
    Connect: Connect,
    Join: Join,
    Editing: QuestionEditor,
    Game: Game
  };

  // Type of the routes object
  type Routes = typeof routes;

  // Extracts the component properties type from the provided type
  type Props<T> = ComponentProps<
    T extends ComponentType<infer Component> ? Component : never
  >;

  // State for the current route
  const routeState: Writable<{ component: ComponentType; props?: object }> =
    writable({
      component: routes.Home,
      props: {}
    });

  /**
   * Sets the current route to the route at the provided key
   * with the provided props
   *
   * @param key   The key for the route defintion
   * @param props The props for the route
   */
  export function setRoute<T extends keyof Routes>(
    key: T,
    props?: Props<Routes[T]>
  ) {
    routeState.set({
      component: routes[key],
      props
    });
  }
</script>

<svelte:component this={$routeState.component} {...$routeState.props} />

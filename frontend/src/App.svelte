<script lang="ts">
  import { AppStateType, createAppState } from "$stores/state.svelte";

  import GlobalDialog from "$components/GlobalDialog.svelte";
  import Loading from "$pages/Loading.svelte";
  import Connect from "$pages/Connect.svelte";
  import Create from "$pages/Create.svelte";
  import Home from "$pages/Home.svelte";
  import Game from "$pages/Game.svelte";
  import stateContext from "$lib/context/state";
  import { createSocketState } from "$lib/stores/socket.svelte";
  import socketContext from "$lib/context/socket";

  const state = createAppState();
  const socket = createSocketState(state);

  const appState = $derived(state.current);

  stateContext.set(state);
  socketContext.set(socket);
</script>

{#if socket.ready}
  {#if appState.ty == AppStateType.Home}
    <Home />
  {:else if appState.ty === AppStateType.Create}
    <Create />
  {:else if appState.ty === AppStateType.Connect}
    <Connect />
  {:else if appState.ty === AppStateType.Game}
    <Game gameData={appState.gameData} />
  {/if}
{:else}
  <Loading text="Connecting to server..." />
{/if}

<GlobalDialog />

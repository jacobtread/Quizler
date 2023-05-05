<script lang="ts">
  import Create from "$pages/Create.svelte";
  import Game from "$pages/Game.svelte";
  import Join from "$pages/Connect.svelte";
  import JoinName from "$pages/Join.svelte";
  import { socketReady } from "$lib/socket";
  import { AppStateType, appState, setConnect, setCreate } from "$stores/state";
</script>

<main class="main">
  {#if $socketReady}
    {#if $appState.ty == AppStateType.Home}
      <button on:click={setCreate}> Create </button>
      <button on:click={setConnect}> Join </button>
    {:else if $appState.ty === AppStateType.Create}
      <Create />
    {:else if $appState.ty === AppStateType.Connect}
      <Join />
    {:else if $appState.ty === AppStateType.Join}
      <JoinName />
    {:else if $appState.ty === AppStateType.Game}
      <Game gameData={$appState.gameData} />
    {/if}
  {:else}
    <p>Connecting to server...</p>
  {/if}
</main>

<style>
</style>

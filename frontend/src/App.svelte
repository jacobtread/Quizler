<script lang="ts">
  import Create from "./lib/Create.svelte";
  import Game from "./lib/Game.svelte";
  import Join from "./lib/Connect.svelte";
  import JoinName from "./lib/Join.svelte";
  import { socketReady } from "./lib/socket";
  import { AppState, appState } from "./lib/state";
</script>

<main class="main">
  {#if $socketReady}
    {#if $appState == AppState.Home}
      <button on:click={() => ($appState = AppState.Create)}> Create </button>
      <button on:click={() => ($appState = AppState.Connect)}> Join </button>
    {:else if $appState === AppState.Create}
      <Create />
    {:else if $appState === AppState.Connect}
      <Join />
    {:else if $appState === AppState.Join}
      <JoinName />
    {:else if $appState === AppState.Game}
      <Game />
    {/if}
  {:else}
    <p>Connecting to server...</p>
  {/if}
</main>

<style>
</style>

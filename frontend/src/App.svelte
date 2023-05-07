<script lang="ts">
  import Create from "$pages/Create.svelte";
  import Game from "$pages/Game.svelte";
  import Join from "$pages/Connect.svelte";
  import JoinName from "$pages/Join.svelte";
  import { socketReady } from "$lib/socket";
  import { AppStateType, appState } from "$stores/state";
  import Home from "$lib/pages/Home.svelte";
  import Loading from "$lib/pages/Loading.svelte";
</script>

{#if $socketReady}
  {#if $appState.ty == AppStateType.Home}
    <Home />
  {:else if $appState.ty === AppStateType.Create}
    <Create />
  {:else if $appState.ty === AppStateType.Connect}
    <Join />
  {:else if $appState.ty === AppStateType.Join}
    <JoinName token={$appState.token} />
  {:else if $appState.ty === AppStateType.Game}
    <Game gameData={$appState.gameData} />
  {/if}
{:else}
  <Loading />
{/if}

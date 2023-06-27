<script lang="ts">
  import { AppStateType, appState } from "$stores/state";
  import { socketReady } from "$api/socket";

  import GlobalDialog from "$components/GlobalDialog.svelte";
  import Loading from "$pages/Loading.svelte";
  import Connect from "$pages/Connect.svelte";
  import Create from "$pages/Create.svelte";
  import Home from "$pages/Home.svelte";
  import Game from "$pages/Game.svelte";
</script>

{#if $socketReady}
  {#if $appState.ty == AppStateType.Home}
    <Home />
  {:else if $appState.ty === AppStateType.Create}
    <Create />
  {:else if $appState.ty === AppStateType.Connect}
    <Connect />
  {:else if $appState.ty === AppStateType.Game}
    <Game gameData={$appState.gameData} />
  {/if}
{:else}
  <Loading text="Connecting to server..." />
{/if}

<GlobalDialog />

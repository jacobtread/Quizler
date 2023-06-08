<script lang="ts">
  import { AppStateType, appState } from "$stores/state";
  import GlobalDialog from "$components/GlobalDialog.svelte";
  import { socketReady } from "$api/socket";
  import Loading from "$pages/Loading.svelte";
  import Connect from "$pages/Connect.svelte";
  import Game from "$pages/Game.svelte";
  import Create from "$pages/Create.svelte";
  import Home from "$pages/Home.svelte";
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

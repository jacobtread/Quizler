<script lang="ts">
  import { writable } from "svelte/store";
  import { GameState } from "./socket/models";
  import { gameHost } from "./socket";

  export const gameState = writable<GameState>(GameState.Lobby);
</script>

<h1>Game</h1>
<p>Host: {$gameHost}</p>
<p>{$gameState}</p>

<style>
</style>

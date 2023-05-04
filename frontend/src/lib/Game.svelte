<script lang="ts">
  import { writable } from "svelte/store";
  import { GameState, type BasicConfig } from "./socket/models";
  import { gameHost } from "./socket";
  import { gameData, gameState } from "./game";
</script>

<h1>Game</h1>
<p>Host: {$gameHost}</p>
<p>{$gameState}</p>
<p>{$gameData.token}</p>

<style>
</style>

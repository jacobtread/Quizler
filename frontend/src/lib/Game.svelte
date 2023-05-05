<script lang="ts">
  import { writable } from "svelte/store";
  import { GameState, type BasicConfig } from "./socket/models";
  import { gameHost } from "./socket";
  import { gameData, gameState, players } from "./game";
</script>

<h1>Game</h1>
<p>Host: {$gameHost}</p>
<p>{$gameState}</p>
<p>{$gameData?.token ?? "Unknown token"}</p>

<ul>
  {#each $players as player}
    <li>{player.name}</li>
  {/each}
</ul>

<style>
</style>

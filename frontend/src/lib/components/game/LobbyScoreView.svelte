<!-- Game view for admin showing the player scores and timer -->
<script lang="ts">
  import {
    type OtherPlayer,
    type SessionId,
    type TimerState
  } from "$lib/socket/models";
  import type { GameData } from "$lib/stores/state";
  import { formatTime } from "$lib/utils";

  export let gameData: GameData;
  export let players: OtherPlayer[];
  export let timer: TimerState;
  export let scores: Record<SessionId, number>;
</script>

<p class="token">
  {gameData.token}
</p>
<p>Remaining: {formatTime(timer)}</p>
<h1>{gameData.config.basic.name}</h1>
<p>{gameData.config.basic.text}</p>

<ul>
  {#each players as player}
    <li>
      <span>{player.name}</span>
      <span>{scores[player.id] ?? 0}</span>
    </li>
  {/each}
</ul>

<style>
  .token {
    color: #ff5500;
    font-weight: bold;
    font-size: 3rem;
    margin: 0;
  }
</style>

<!-- Game view for admin showing the player scores and timer -->
<script lang="ts">
  import {
    ClientMessage,
    HostAction,
    type OtherPlayer,
    type SessionId,
    type TimerState,
    ServerError
  } from "$lib/socket/models";
  import * as socket from "$lib/socket";
  import type { GameData } from "$lib/stores/state";
  import { formatTime } from "$lib/utils";

  export let gameData: GameData;
  export let players: OtherPlayer[];
  export let timer: TimerState;
  export let scores: Record<SessionId, number>;

  async function doSkip() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Skip
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to cancel", error);
    }
  }
</script>

<p class="token">
  {gameData.token}
</p>
<p>Remaining: {formatTime(timer)}</p>
<h1>{gameData.config.name}</h1>
<p>{gameData.config.text}</p>
<button on:click={doSkip}>Skip Countdown</button>

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

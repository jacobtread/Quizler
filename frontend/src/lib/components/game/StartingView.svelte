<!-- Game view when the state is in the "Starting" -->
<script lang="ts">
  import * as socket from "$lib/socket";
  import {
    ClientMessage,
    HostAction,
    type TimerState,
    ServerError
  } from "$lib/socket/models";
  import type { GameData } from "$lib/stores/state";
  import { formatTime } from "$lib/utils";

  export let gameData: GameData;
  export let timer: TimerState;

  async function doCancel() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Cancel
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to cancel", error);
    }
  }

  async function doSkip() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Skip
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to skip", error);
    }
  }
</script>

<h1>
  {gameData.config.name}
  {#if gameData.host}
    <span class="host">Host</span>
  {/if}
</h1>
<p>{gameData.config.text}</p>

<h1>Starting</h1>
<p>Countdown: {formatTime(timer)}</p>

{#if gameData.host}
  <button on:click={doSkip}>Skip Countdown</button>
  <button on:click={doCancel}>Cancel</button>
{/if}

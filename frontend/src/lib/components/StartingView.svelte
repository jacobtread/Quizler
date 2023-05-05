<!-- Game view when the state is in the "Starting" -->
<script lang="ts">
  import { sendMessage } from "$lib/socket";
  import {
    ClientMessageType,
    ServerMessage,
    HostAction,
    type TimerState
  } from "$lib/socket/models";
  import type { GameData } from "$lib/stores/state";
  import { formatTime } from "$lib/utils";

  export let gameData: GameData;
  export let timer: TimerState;

  async function doCancel() {
    let res = await sendMessage({
      ty: ClientMessageType.HostAction,
      action: HostAction.Cancel
    });

    if (res.ty === ServerMessage.Error) {
      console.error("Error while attempting to cancel", res.error);
    }
  }

  async function doSkip() {
    let res = await sendMessage({
      ty: ClientMessageType.HostAction,
      action: HostAction.Skip
    });

    if (res.ty === ServerMessage.Error) {
      console.error("Error while attempting to cancel", res.error);
    }
  }
</script>

<h1>
  {gameData.config.basic.name}
  {#if gameData.host}
    <span class="host">Host</span>
  {/if}
</h1>
<p>{gameData.config.basic.text}</p>

<h1>Starting</h1>
<p>Countdown: {formatTime(timer)}</p>

{#if gameData.host}
  <button on:click={doSkip}>Skip Countdown</button>
  <button on:click={doCancel}>Cancel</button>
{/if}

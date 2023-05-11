<script lang="ts">
  import * as socket from "$lib/socket";
  import { ClientMessage, HostAction, ServerError } from "$lib/socket/models";
  import type { GameData } from "$lib/stores/state";

  export let gameData: GameData;

  async function doReset() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Reset
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to reset", error);
    }
  }
</script>

{#if gameData.host}
  <button on:click={doReset}> Restart </button>
{/if}

<h1>Game Over</h1>
<!-- TODO: top players list -->

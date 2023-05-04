<script lang="ts">
  import { gameHost, getSocketReady, sendMessage } from "./socket";
  import { ClientMessageType } from "./socket/models";
  import { AppState, appState } from "./state";

  let name = "";

  async function joinQuiz() {
    // TODO: name validation

    // Await the socket being alive
    await getSocketReady();

    gameHost.set(false);

    sendMessage({
      ty: ClientMessageType.Join,
      name
    });
  }
</script>

<button on:click={() => ($appState = AppState.Home)}>Back</button>
<p>Enter name to join as</p>

<label for="">
  Name
  <input type="text" bind:value={name} />
</label>

<button on:click={joinQuiz}>Join</button>

<style>
</style>

<script lang="ts">
  import { getSocketReady, sendMessage } from "./socket";
  import { ClientMessageType, ServerMessage } from "./socket/models";
  import { AppState, appState } from "./state";

  let token: string = "";

  async function connectQuiz() {
    // TODO: proper token validation
    if (token.length < 5) {
      return;
    }

    // Await the socket being alive
    await getSocketReady();

    const resp = await sendMessage({
      ty: ClientMessageType.Connect,
      token
    });

    if (resp.ty === ServerMessage.Error) {
      console.error("Error while initializing", resp.error);
    } else {
      appState.set(AppState.Join);
    }
  }
</script>

<button on:click={() => ($appState = AppState.Home)}>Back</button>
<p>Join Game</p>
<label for="">
  Code
  <input type="text" bind:value={token} />
</label>
<button on:click={connectQuiz}>Connect</button>

<style>
</style>

<script lang="ts">
  import { gameData } from "./game";
  import { gameHost, getSocketReady, sendMessage } from "./socket";
  import { ClientMessageType, ServerMessage } from "./socket/models";
  import { AppState, appState } from "./state";

  let name = "";

  async function joinQuiz() {
    // TODO: name validation

    // Await the socket being alive
    await getSocketReady();

    gameHost.set(false);

    const resp = await sendMessage({
      ty: ClientMessageType.Join,
      name
    });

    if (resp.ty === ServerMessage.Error) {
      console.error("Error while initializing", resp.error);
    } else {
      const { id, token, config } = resp;

      gameData.set({ id, token, config });
      appState.set(AppState.Game);
    }
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

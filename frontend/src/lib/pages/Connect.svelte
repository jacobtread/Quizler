<script lang="ts">
  import { getSocketReady, sendMessage } from "$lib/socket";
  import { ClientMessageType, ServerMessage } from "$lib/socket/models";
  import { setHome, setJoin } from "$stores/state";

  let token: string = "";

  async function connectQuiz() {
    // TODO: proper token validation
    if (token.length < 5) {
      return;
    }

    // Await the socket being alive
    await getSocketReady();

    const resp = await sendMessage(ClientMessageType.Connect, {
      token
    });

    if (resp.ty === ServerMessage.Error) {
      console.error("Error while initializing", resp.error);
    } else {
      setJoin(token);
    }
  }
</script>

<button on:click={setHome}>Back</button>
<p>Join Game</p>
<label for="">
  Code
  <input type="text" bind:value={token} />
</label>
<button on:click={connectQuiz}>Connect</button>

<style>
</style>

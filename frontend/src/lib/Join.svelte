<script lang="ts">
  import { getSocketReady, sendMessage } from "./socket";
  import { ClientMessageType, ServerMessage } from "./socket/models";
  import { setGame, setHome } from "./state";

  let name = "";

  async function joinQuiz() {
    // TODO: name validation

    // Await the socket being alive
    await getSocketReady();

    const resp = await sendMessage({
      ty: ClientMessageType.Join,
      name
    });

    if (resp.ty === ServerMessage.Error) {
      console.error("Error while initializing", resp.error);
    } else {
      const { id, token, config } = resp;

      setGame({
        id,
        token,
        config,
        host: false
      });
    }
  }
</script>

<button on:click={setHome}>Back</button>
<p>Enter name to join as</p>

<label for="">
  Name
  <input type="text" bind:value={name} />
</label>

<button on:click={joinQuiz}>Join</button>

<style>
</style>

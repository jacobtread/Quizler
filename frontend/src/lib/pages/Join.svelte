<script lang="ts">
  import * as socket from "$lib/socket";
  import { ClientMessage, ServerMessage } from "$lib/socket/models";
  import { setGame, setHome } from "$stores/state";

  export let token: string;

  let name = "";

  async function joinQuiz() {
    // TODO: name validation

    // Await the socket being alive
    await socket.ready();

    const resp = await socket.send({
      ty: ClientMessage.Join,
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
<h2>{token}</h2>

<p>Enter name to join as</p>

<label for="">
  Name
  <input type="text" bind:value={name} />
</label>

<button on:click={joinQuiz}>Join</button>

<style>
</style>

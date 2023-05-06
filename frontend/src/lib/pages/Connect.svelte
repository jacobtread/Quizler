<script lang="ts">
  import { TOKEN_LENGTH } from "$lib/constants";
  import * as socket from "$lib/socket";
  import { ClientMessage, ServerMessage } from "$lib/socket/models";
  import { setHome, setJoin } from "$stores/state";
  import { z } from "zod";

  let userToken: string = "";
  let disabled: boolean = true;

  const tokenSchema = z
    .string()
    .toUpperCase()
    .length(TOKEN_LENGTH)
    .regex(/[A-Z0-9]/);

  function onTokenInput() {
    userToken = userToken
      // Convert the value to uppercase format
      .toUpperCase()
      // Remove any invald values
      .replace(/[^A-Z0-9]/, "");

    // Change the disabled state
    disabled = !tokenSchema.safeParse(userToken).success;
  }

  async function connectQuiz() {
    const parse = tokenSchema.safeParse(userToken);

    if (!parse.success) {
      return;
    }

    const token = parse.data;

    // Await the socket being alive
    await socket.ready();

    const resp = await socket.send({
      ty: ClientMessage.Connect,
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
  <input
    type="text"
    bind:value={userToken}
    on:input={onTokenInput}
    minlength={TOKEN_LENGTH}
    maxlength={TOKEN_LENGTH}
  />
</label>
<button on:click={connectQuiz} {disabled}>Connect</button>

<style>
</style>

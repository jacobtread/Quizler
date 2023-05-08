<script lang="ts">
  import * as socket from "$lib/socket";
  import { ClientMessage, ServerMessage, errorText } from "$lib/socket/models";
  import { errorDialog } from "$lib/stores/dialogStore";
  import { setConnect, setGame } from "$stores/state";
  import { z } from "zod";
  import Back from "$lib/assets/icons/back.svg";

  export let token: string;

  let name = "";

  const nameSchema = z
    .string()
    .min(1, "Name cannot be empty")
    .max(30, "Name cannot be more than 30 characters long");
  let disabled: boolean = true;

  function onNameInput() {
    // Change the disabled state
    disabled = !nameSchema.safeParse(name).success;
  }

  async function joinQuiz() {
    const parse = nameSchema.safeParse(name);

    if (!parse.success) {
      console.error("Failed to parse name", parse.error);
      return;
    }

    // Await the socket being alive
    await socket.ready();

    const resp = await socket.send({
      ty: ClientMessage.Join,
      name
    });

    if (resp.ty === ServerMessage.Error) {
      console.error("Error while joining", resp.error);
      errorDialog("Failed to join", errorText[resp.error]);
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

<button on:click={setConnect} class="back back--floating">
  <img src={Back} alt="Back" />
</button>

<h2>{token}</h2>

<p>Enter name to join as</p>

<label for="">
  Name
  <input type="text" bind:value={name} on:input={onNameInput} />
</label>

<button on:click={joinQuiz} {disabled}>Join</button>

<style>
</style>

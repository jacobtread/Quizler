<script lang="ts">
  import * as socket from "$lib/socket";
  import { ClientMessage, ServerError, errorText } from "$lib/socket/models";
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

    try {
      const { id, token, config } = await socket.send({
        ty: ClientMessage.Join,
        name
      });

      setGame({ id, token, config, host: false });
    } catch (e) {
      const error = e as ServerError;
      console.error("Failed to join", error);
      errorDialog("Failed to join", errorText[error]);
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

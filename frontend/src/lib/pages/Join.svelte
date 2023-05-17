<script lang="ts">
  import { slide } from "svelte/transition";
  import { z } from "zod";

  import * as socket from "$lib/socket";
  import { ClientMessage, ServerError, errorText } from "$lib/socket/models";

  import { errorDialog } from "$stores/dialogStore";
  import { setGame, setConnect } from "$stores/state";

  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";

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

      setGame({ id, token, config, host: false, name });
    } catch (e) {
      const error = e as ServerError;
      console.error("Failed to join", error);
      errorDialog("Failed to join", errorText[error]);
    }
  }
</script>

<main class="main" transition:slide>
  <button on:click={setConnect} class="back back--floating">
    <Back />
  </button>
  <h2>{token}</h2>

  <h1>Enter Name</h1>

  <p>Please enter your desired name</p>
  <div class="form">
    <input
      class="input"
      type="text"
      bind:value={name}
      on:input={onNameInput}
      minlength={1}
      maxlength={30}
    />

    {#if !disabled}
      <button
        on:click={joinQuiz}
        class="play"
        transition:slide={{ axis: "x", duration: 200 }}
      >
        <Play />
      </button>
    {/if}
  </div>
</main>

<style lang="scss">
  @import "../../assets/scheme";

  .main {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    flex-flow: column;
    height: 100%;
  }

  .form {
    max-width: 32rem;
    width: 100%;
    padding: 1rem;
    text-align: center;
  }

  .input {
    display: inline-block;
    padding: 0.7rem;
    font-size: 2rem;
    width: 100%;
    max-width: 15rem;
    text-align: center;
    background-color: transparent;
    border: 5px solid #222;
    color: #fff;
    border-radius: 0.5rem;
    outline: none;
    transition: 0.5s ease;
    letter-spacing: 0.25rem;
    vertical-align: middle;
  }

  .input:focus {
    border-bottom-color: #f66828;
  }

  .play {
    vertical-align: middle;
    margin-left: 1rem;
    padding: 0.6rem;
    border-radius: 1rem;
    border: none;
    cursor: pointer;
    color: #fff;
    background-color: #f66828;
  }

  @media screen and (max-width: 32rem) {
    .play,
    .input {
      max-width: none;
      display: block;
      margin: 1rem auto;
      width: 100%;
    }
  }
</style>

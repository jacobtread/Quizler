<script lang="ts">
  import { slide } from "svelte/transition";
  import { z } from "zod";

  import * as socket from "$lib/socket";
  import { ClientMessage, ServerError, errorText } from "$lib/socket/models";

  import { errorDialog } from "$stores/dialogStore";

  import { setRoute } from "$components/Router.svelte";

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

      setRoute("Game", {
        gameData: { id, token, config, host: false, name }
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Failed to join", error);
      errorDialog("Failed to join", errorText[error]);
    }
  }

  const back = () => setRoute("Connect");
</script>

<main class="main" transition:slide>
  <button on:click={back} class="back back--floating">
    <Back />
  </button>
  <h2>{token}</h2>

  <h1>Enter Name</h1>

  <p>Please enter your desired name</p>
  <div class="form">
    <label>
      <input
        class="input"
        type="text"
        bind:value={name}
        on:input={onNameInput}
        minlength={1}
        maxlength={30}
      />
    </label>

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

<style>
  .main {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    flex-flow: column;
    height: 100%;
  }

  .form {
    display: grid;
    gap: 1rem;
    grid-template-columns: auto min-content;
  }

  .input {
    display: block;
    padding: 0.7rem;
    font-size: 2rem;
    width: 100%;
    max-width: 24rem;
    text-align: left;
    background-color: transparent;
    border: 5px solid #222;
    color: #fff;
    border-radius: 0.5rem;
    outline: none;
    transition: 0.5s ease;
    letter-spacing: 0.25rem;
  }

  .input:focus {
    border-bottom-color: #f66828;
  }

  .play {
    padding: 0.5rem;
    border-radius: 1rem;
    border: none;
    cursor: pointer;
    color: #fff;
    background-color: #f66828;
  }
</style>

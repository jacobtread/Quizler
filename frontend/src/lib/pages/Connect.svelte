<script lang="ts">
  import { TOKEN_LENGTH } from "$lib/constants";
  import * as socket from "$lib/socket";
  import { ClientMessage, ServerMessage, errorText } from "$lib/socket/models";
  import { setHome, setJoin } from "$stores/state";
  import { z } from "zod";
  import Play from "$lib/assets/icons/play.svg";
  import { slide } from "svelte/transition";
  import { errorDialog } from "$lib/stores/dialogStore";

  let userToken: string = "";
  let disabled: boolean = true;

  const tokenSchema = z
    .string()
    .toUpperCase()
    .length(TOKEN_LENGTH, "Invalid token length")
    .regex(/^[A-Z0-9]+$/, "Token didn't match token charset");

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
      console.error("Failed to parse token", parse.error);
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
      console.error("Error while connecting", resp.error);
      errorDialog("Failed to connect", errorText[resp.error]);
    } else {
      setJoin(token);
    }
  }
</script>

<main class="main">
  <button on:click={setHome} class="back">Back</button>

  <h1>Enter Code</h1>
  <p>Please enter your quiz code below</p>
  <div class="form">
    <label>
      <input
        class="input"
        type="text"
        bind:value={userToken}
        on:input={onTokenInput}
        minlength={TOKEN_LENGTH}
        maxlength={TOKEN_LENGTH}
        placeholder={"X".repeat(TOKEN_LENGTH)}
      />
    </label>

    {#if !disabled}
      <button
        on:click={connectQuiz}
        class="play"
        transition:slide={{ axis: "x", duration: 200 }}
      >
        <img src={Play} alt="Play Icon" class="path__icon" />
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

  .back {
    position: absolute;
    left: 0.5rem;
    top: 0.5rem;
  }

  .play {
    padding: 1rem;
    border-radius: 1rem;
    border: none;
    cursor: pointer;
    color: #fff;
    background-color: #f66828;
  }

  .input {
    flex: auto;
    display: block;
    padding: 0.7rem;
    font-size: 3rem;
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
  }

  .input:focus {
    border-bottom-color: #f66828;
  }

  .form {
    display: flex;
    gap: 1rem;
  }
</style>

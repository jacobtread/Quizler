<script lang="ts">
  import { slide } from "svelte/transition";

  import {
    MAX_PLAYER_NAME_LENGTH,
    MIN_PLAYER_NAME_LENGTH
  } from "$lib/constants";

  import * as socket from "$lib/socket";
  import { ClientMessage, ServerError, errorText } from "$lib/socket/models";

  import { errorDialog } from "$stores/dialogStore";
  import { setGame, setConnect } from "$stores/state";

  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";

  // The token of the game that is being joined
  export let token: string;

  // The user provided name
  let name = "";

  // Disabled state for the join button
  let disabled: boolean = true;

  /**
   * Updates the disabled state after validating the
   * provided player name
   */
  function updateName() {
    // Change the disabled state based on the name lengthh
    disabled =
      name.length >= MIN_PLAYER_NAME_LENGTH &&
      name.length <= MAX_PLAYER_NAME_LENGTH;
  }

  function join() {
    socket
      .send({ ty: ClientMessage.Join, name })
      .then(({ id, token, config }) => {
        setGame({ id, token, config, host: false, name });
      })
      .catch((error: ServerError) => {
        console.error("Failed to join", error);
        errorDialog("Failed to join", errorText[error]);
      });
  }
</script>

<main class="main" transition:slide>
  <button on:click={setConnect} class="back back--floating">
    <Back />
  </button>

  <p>{token}</p>
  <h1>Enter Name</h1>
  <p>Please enter your desired name</p>

  <div class="form">
    <input
      class="input"
      type="text"
      bind:value={name}
      on:input={updateName}
      minlength={MIN_PLAYER_NAME_LENGTH}
      maxlength={MAX_PLAYER_NAME_LENGTH}
    />

    {#if !disabled}
      <button
        on:click={join}
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

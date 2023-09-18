<script lang="ts">
  import { afterUpdate } from "svelte";
  import { slide } from "svelte/transition";

  import {
    MAX_PLAYER_NAME_LENGTH,
    MIN_PLAYER_NAME_LENGTH,
    TOKEN_LENGTH
  } from "$lib/constants";

  import * as socket from "$api/socket";
  import { ClientMessage, ServerError, errorText } from "$api/models";

  import FloatingLoader from "$components/FloatingLoader.svelte";
  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";

  import { setGame, setHome } from "$stores/state";
  import { errorDialog } from "$stores/dialogStore";

  const enum State {
    Connect,
    Join
  }

  let state = State.Connect;

  // The user provided token
  let token: string = "";

  // The user provided name
  let name = "";

  // Disabled state for the connect button
  let tokenValid: boolean = false;

  // Disabled state for the connect button
  let nameValid: boolean = false;

  // Loading screen state
  let loading: boolean = false;

  // Determines whether focus should be updated
  let updateFocus: boolean = true;

  let inputToken: HTMLInputElement;
  let inputName: HTMLInputElement;

  /**
   * Update called whenever the token input changes in
   * order to normalize the casing of the input along
   * with changing the disabled state based on the
   * length requirement
   */
  function updateToken() {
    token = token
      // Convert the value to uppercase format
      .toUpperCase()
      // Remove any invald values
      .replace(/[^A-Z0-9]/, "");

    // Enforce max token length for mobile devices
    if (token.length > TOKEN_LENGTH) {
      token = token.substring(0, TOKEN_LENGTH);
    }

    // Change the disabled state based on the length requirement
    tokenValid = token.length === TOKEN_LENGTH;
  }

  /**
   * Updates the disabled state after validating the
   * provided player name
   */
  function updateName() {
    // Change the disabled state based on the name lengthh
    nameValid =
      name.length >= MIN_PLAYER_NAME_LENGTH &&
      name.length <= MAX_PLAYER_NAME_LENGTH;
  }

  /**
   * Handles attempting to connect to a quiz using the
   * user provided token.
   */
  function connect() {
    loading = true;

    socket
      .send({ ty: ClientMessage.Connect, token })
      .then(() => {
        state = State.Join;
        updateFocus = true;
      })
      .catch((error: ServerError) => {
        console.error("Failed to connect", error);
        errorDialog("Failed to connect", errorText[error]);
      })
      .finally(() => (loading = false));
  }

  function join() {
    loading = true;

    socket
      .send({ ty: ClientMessage.Join, name })
      .then(({ id, token, config }) => {
        setGame({ id, token, config, host: false, name });
      })
      .catch((error: ServerError) => {
        console.error("Failed to join", error);
        errorDialog("Failed to join", errorText[error]);
      })
      .finally(() => (loading = false));
  }

  function back() {
    if (state === State.Connect) {
      setHome();
    } else {
      state = State.Connect;
      inputToken.focus();
    }
  }

  afterUpdate(() => {
    if (!updateFocus) return;

    if (inputToken && document.activeElement !== inputToken) {
      inputToken.focus();
    }

    if (inputName && document.activeElement !== inputName) {
      inputName.focus();
    }
  });
</script>

{#if loading} <FloatingLoader /> {/if}

<button on:click={back} class="back back--floating">
  <Back />
</button>

{#if state === State.Connect}
  <main class="page page--center page--overflow" transition:slide|global>
    <h1>Enter Code</h1>
    <p>Please enter your quiz code below</p>

    <form class="form" on:submit|preventDefault={connect}>
      <input
        bind:this={inputToken}
        class="special-input"
        type="text"
        bind:value={token}
        on:input={updateToken}
        minlength={TOKEN_LENGTH}
        maxlength={TOKEN_LENGTH}
        placeholder={"X".repeat(TOKEN_LENGTH)}
      />

      {#if tokenValid}
        <button
          type="submit"
          class="play"
          transition:slide={{ axis: "x", duration: 200 }}
        >
          <Play />
        </button>
      {/if}
    </form>
  </main>
{:else}
  <main class="page page--center page--overflow" transition:slide|global>
    <p>{token}</p>
    <h1>Enter Name</h1>
    <p>Please enter your desired name</p>

    <form class="form" on:submit|preventDefault={join}>
      <input
        bind:this={inputName}
        class="special-input special-input--small"
        type="text"
        bind:value={name}
        on:input={updateName}
        minlength={MIN_PLAYER_NAME_LENGTH}
        maxlength={MAX_PLAYER_NAME_LENGTH}
      />

      {#if nameValid}
        <button
          type="submit"
          class="play play--small"
          transition:slide={{ axis: "x", duration: 200 }}
        >
          <Play />
        </button>
      {/if}
    </form>
  </main>
{/if}

<style lang="scss">
  @import "../../assets/scheme";

  .page {
    gap: 1rem;
  }

  .form {
    max-width: 32rem;
    width: 100%;
    padding: 1rem;
    text-align: center;
  }

  .special-input {
    display: inline-block;
    padding: 0.7rem;
    font-size: 3rem;
    width: 100%;
    max-width: 15rem;
    text-align: center;
    background-color: transparent;
    border: 5px solid $surface;
    color: $textPrimary;
    border-radius: 0.5rem;
    outline: none;
    transition: 0.5s ease;
    letter-spacing: 0.25rem;
    vertical-align: middle;

    &:focus {
      border-bottom-color: $primary;
    }

    &--small {
      font-size: 2rem;
    }
  }

  .play {
    vertical-align: middle;
    margin-left: 1rem;
    padding: 0.8rem;
    border-radius: 1rem;
    border: none;
    cursor: pointer;
    color: #fff;
    background-color: $primary;

    &--small {
      padding-inline-start: 0.6rem;
    }
  }

  @media screen and (max-width: 32rem) {
    .play,
    .special-input {
      max-width: none;
      display: block;
      margin: 1rem auto;
      width: 100%;
    }
  }
</style>

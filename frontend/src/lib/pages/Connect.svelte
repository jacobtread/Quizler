<script lang="ts">
  import { slide } from "svelte/transition";

  import { TOKEN_LENGTH } from "$lib/constants";

  import * as socket from "$lib/socket";
  import { ClientMessage, ServerError, errorText } from "$lib/socket/models";

  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";

  import { setHome, setJoin } from "$stores/state";
  import { errorDialog } from "$stores/dialogStore";

  // The user provided token
  let token: string = "";

  // Disabled state for the connect button
  let disabled: boolean = true;

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

    // Change the disabled state based on the length requirement
    disabled = token.length != TOKEN_LENGTH;
  }

  /**
   * Handles attempting to connect to a quiz using the
   * user provided token.
   */
  function connect() {
    socket
      .send({ ty: ClientMessage.Connect, token })
      .then(() => setJoin(token))
      .catch((error: ServerError) => {
        console.error("Failed to connect", error);
        errorDialog("Failed to connect", errorText[error]);
      });
  }
</script>

<main class="main" transition:slide>
  <button on:click={setHome} class="back back--floating">
    <Back />
  </button>

  <h1>Enter Code</h1>
  <p>Please enter your quiz code below</p>
  <div class="form">
    <input
      class="input"
      type="text"
      bind:value={token}
      on:input={updateToken}
      minlength={TOKEN_LENGTH}
      maxlength={TOKEN_LENGTH}
      placeholder={"X".repeat(TOKEN_LENGTH)}
    />

    {#if !disabled}
      <button
        on:click={connect}
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
    font-size: 3rem;
    width: 100%;
    max-width: 15rem;
    text-align: center;
    background-color: transparent;
    border: 5px solid $surface;
    color: #fff;
    border-radius: 0.5rem;
    outline: none;
    transition: 0.5s ease;
    letter-spacing: 0.25rem;
    vertical-align: middle;

    &:focus {
      border-bottom-color: $primary;
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

<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import { NameFiltering } from "$api/models";
  import * as constants from "$lib/constants";

  import { createData } from "$stores/createStore";

  export let visible: boolean;
</script>

<div class="floating-wrapper">
  <div class="dialog">
    <button
      on:click={() => (visible = false)}
      class="btn btn--icon btn--surface"
    >
      <Close />
      Close
    </button>
    <label class="field">
      <span class="field__name">Title</span>
      <p class="field__desc">
        Give your quiz a title <span class="optional">Optional</span>
      </p>
      <input
        class="input"
        type="text"
        bind:value={$createData.name}
        maxlength={constants.MAX_TITLE_LENGTH}
      />
    </label>
    <label class="field">
      <span class="field__name">Description</span>
      <p class="field__desc">
        Description of your Quiz <span class="optional">Optional</span>
      </p>
      <textarea
        class="input input--desc"
        name=""
        id=""
        cols="30"
        rows="5"
        bind:value={$createData.text}
        maxlength={constants.MAX_DESCRIPTION_LENGTH}
      />
    </label>

    <label class="field">
      <span class="field__name">Max Players</span>
      <p class="field__desc">
        Maximum number of players allowed to join this quiz
      </p>
      <input
        class="input"
        type="number"
        bind:value={$createData.max_players}
        min={constants.MIN_MAX_PLAYERS}
        max={constants.MAX_MAX_PLAYERS}
      />
    </label>
    <label class="field">
      <span class="field__name">Name Filtering</span>
      <p class="field__desc">
        Level of filtering on profane/inappropriate naming. Its recommended that
        you leave this on Medium or High
      </p>
      <select bind:value={$createData.filtering} class="input">
        <option value={NameFiltering.None}>None: Don't filter names</option>
        <option value={NameFiltering.Low}
          >Low: Filter out more severe names</option
        >
        <option value={NameFiltering.Medium}>
          Medium: Filter out anything thats not mild
        </option>
        <option value={NameFiltering.High}>
          High: Filter out as much as possible
        </option>
      </select>
    </label>
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .floating-wrapper {
    z-index: 1;
  }
  @media screen and (max-width: 32rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
  }
  .dialog {
    background-color: $surface;

    border-radius: 0.5rem;

    width: 100%;
    max-width: 32rem;

    margin: 1rem;
    padding: 1rem;

    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  .optional {
    color: #777;
    margin-left: 0.5rem;
  }

  .field {
    display: block;
    background-color: $surface;
    border-radius: 0.55rem;

    &__name {
      font-weight: bold;
      color: #ffffff;
    }

    &__desc {
      color: #cccccc;
      margin-bottom: 0.25rem;
    }
  }

  .input {
    display: block;
    margin-top: 0.25rem;
    width: 100%;
    padding: 0.5rem;
    border: none;
    background-color: $surfaceLight;
    border-radius: 0.25rem;
    margin-top: 0.5rem;
    font-size: 1rem;
    line-height: 1.5;
  }
</style>

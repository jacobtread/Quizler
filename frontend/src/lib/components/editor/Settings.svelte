<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import { NameFiltering } from "$api/models";
  import * as constants from "$lib/constants";

  import { createData } from "$stores/createStore";
  import { fade, slide } from "svelte/transition";

  export let visible: boolean;
</script>

<div class="floating-wrapper" transition:fade={{ duration: 200 }}>
  <div class="dialog" transition:slide={{ duration: 200 }}>
    <button
      on:click={() => (visible = false)}
      class="btn btn--icon btn--surface"
    >
      <Close />
      Close
    </button>

    <div class="section">
      <div class="section__values">
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
            cols="40"
            rows="2"
            bind:value={$createData.text}
            maxlength={constants.MAX_DESCRIPTION_LENGTH}
          />
        </label>
      </div>
    </div>

    <div class="section">
      <div class="section__values">
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
            Level of filtering on profane/inappropriate naming. Its recommended
            that you leave this on <b>Medium</b> or <b>High</b>
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
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .floating-wrapper {
    z-index: 1;
  }

  .dialog {
    background-color: $surface;

    border-radius: 0.5rem;

    width: 100%;
    max-width: 48rem;

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

  .input--desc {
    resize: none;
  }

  @media screen and (max-width: 32rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
  }
</style>

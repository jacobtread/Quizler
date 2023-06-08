<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import { ImageFit, type Question } from "$api/models";

  export let question: Question;
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

    {#if question.image !== null}
      <label class="field">
        <span class="field__name">Image Fit</span>
        <p class="field__desc">
          Decide how the image should be best fit for the player screens, It's
          recommended that you use "Contain" if the full image content is
          important to be visible
        </p>
        <select class="input" bind:value={question.image.fit}>
          <option value={ImageFit.Contain}>Contain: Fit the entire image</option
          >
          <option value={ImageFit.Cover}>Cover: Fill the available space</option
          >
          <option value={ImageFit.Width}>
            Fill Width: Fill available width
          </option>
          <option value={ImageFit.Height}>
            Fill Height: Fill available height
          </option>
        </select>
      </label>
    {/if}
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .floating-wrapper {
    z-index: 1;
  }

  @media screen and (max-width: 48rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
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

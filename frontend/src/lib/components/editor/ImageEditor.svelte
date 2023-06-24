<script lang="ts">
  import { ImageFit, imageFitText, type Question } from "$lib/api/models";
  import { imagePreviewStore, selectImage } from "$lib/stores/imageStore";
  import ImageStorage from "$components/ImageStorage.svelte";
  import Cog from "../icons/Cog.svelte";
  import FloatingModal, { ModelSize } from "../FloatingModal.svelte";

  export let question: Question;

  let image: string | null = null;
  let settings: boolean = false;

  $: if (question.image !== null) {
    // Handle displaying image previews
    let imagePreview = $imagePreviewStore[question.image.uuid];
    if (imagePreview !== undefined) {
      image = imagePreview;
    } else {
      image = null;
    }
  }

  async function pickImage() {
    let res = await selectImage();
    // Handle canceling select image
    if (res === null) return;

    question.image = {
      uuid: res.uuid,
      fit: ImageFit.Contain
    };
  }

  function removeImage() {
    question.image = null;
    image = null;
  }
</script>

{#if question.image !== null}
  <div class="wrapper">
    <!-- Actual preview image may not be immediately available -->
    {#if image !== null}
      <img
        class="image"
        data-fit={question.image.fit}
        src={image}
        alt="Question Preview"
      />
    {/if}
    <button class="overlay" on:click={removeImage}>Click to remove</button>
    <button
      class="btn btn--icon btn--icon-only settings"
      on:click={() => (settings = true)}
    >
      <Cog />
    </button>
  </div>

  <!-- Dialog for changing image settings -->
  <FloatingModal bind:visible={settings} size={ModelSize.Small}>
    <label class="section">
      <h2 class="section__title">Image Fit</h2>
      <p class="section__desc">
        How the image should be fit to devices. It's recommended that you use
        <b>Contain</b> if its important that they can see the whole image
      </p>

      <select class="input" bind:value={question.image.fit}>
        {#each Object.values(ImageFit) as value}
          <option {value}>{value}: {imageFitText[value]}</option>
        {/each}
      </select>
    </label>
  </FloatingModal>
{:else}
  <div class="wrapper">
    <button class="add" on:click={pickImage}>Pick Image</button>
  </div>
{/if}

<!-- Image store access for rendering the image picker -->
<ImageStorage />

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .settings {
    position: absolute;
    right: 0;
    top: 0;
  }

  .wrapper {
    width: 100%;
    flex: auto;
    overflow: hidden;
    position: relative;
    margin-bottom: 1rem;
  }

  @media screen and (max-width: 64rem) {
    .wrapper {
      height: 50vh;
    }
  }

  .image {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    aspect-ratio: auto;
    z-index: 0;

    // Fit for width
    &[data-fit="Width"] {
      width: 100%;
    }

    // Fit for height
    &[data-fit="Height"] {
      height: 100%;
    }

    // Fit for containing whole image
    &[data-fit="Contain"] {
      height: 100%;
      width: 100%;
      object-fit: contain;
    }

    // Fit for covering available space
    &[data-fit="Cover"] {
      height: 100%;
      width: 100%;
      object-fit: cover;
    }
  }

  .add {
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    font-size: 1rem;
    background-color: transparent;
    border: none;
  }

  .overlay {
    cursor: pointer;
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    transition: opacity 0.15s ease;
    font-size: 1rem;
    background-color: rgba($color: #000000, $alpha: 0.7);
    border: none;
    opacity: 0;

    &:hover {
      opacity: 1;
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

<script lang="ts">
  import { ImageFit, type Question } from "$lib/api/models";
  import { imagePreviewStore, selectImage } from "$lib/stores/imageStore";
  import ImageStorage from "$components/ImageStorage.svelte";
  import Cog from "../icons/Cog.svelte";
  import ImageSettings from "./ImageSettings.svelte";
  import Image from "../icons/Image.svelte";

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

  function removeImage(event: Event) {
    event.stopPropagation();
    question.image = null;
    image = null;
  }
</script>

<div tabindex="0" role="button" class="wrapper">
  {#if question.image !== null && image}
    <img
      class="image"
      data-fit={question.image.fit}
      src={image}
      alt="Uploaded Content"
    />

    <button class="overlay" on:click={removeImage}> Click to remove </button>
    <button
      class="btn btn--icon btn--icon-only settings"
      on:click={() => (settings = true)}
    >
      <Cog />
    </button>
  {:else}
    <button class="add" on:click={pickImage}>Pick Image</button>
  {/if}
</div>

{#if settings}
  <ImageSettings bind:question bind:visible={settings} />
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
    max-height: 50vh;
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
</style>

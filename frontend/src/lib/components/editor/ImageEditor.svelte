<script lang="ts">
  import { ImageFit, type Question } from "$lib/api/models";
  import { imagePreviewStore, selectImage } from "$lib/stores/imageStore";
  import ImageStorage from "$components/ImageStorage.svelte";
  import Cog from "../icons/Cog.svelte";
  import ImageSettings from "./ImageSettings.svelte";

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

  function onImageKeyPress(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === "NumpadEnter") {
      pickImage();
    }
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
    <button class="overlay" on:click={pickImage} on:keypress={onImageKeyPress}
      >Pick Image</button
    >
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

  .overlay {
    position: absolute;
    left: 0;
    top: 0;
    opacity: 0;
    width: 100%;
    height: 100%;
    transition: opacity 0.15s ease;
    font-size: 1rem;
    background-color: rgba($color: #000000, $alpha: 0.7);
    border: none;

    &:hover {
      opacity: 1;
    }
  }

  .field {
    display: block;
    margin-bottom: 1rem;
    background-color: $surface;
    padding: 1rem;
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

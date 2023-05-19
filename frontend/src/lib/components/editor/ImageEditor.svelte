<script lang="ts">
  import type { Question } from "$lib/api/models";
  import { imagePreviewStore, selectImage } from "$lib/stores/imageStore";
  import ImageStorage from "$components/ImageStorage.svelte";

  export let question: Question;

  let image: string | null = null;

  $: {
    // Handle displaying image previews
    if (question.image !== null) {
      let imagePreview = $imagePreviewStore[question.image];
      if (imagePreview !== undefined) {
        image = imagePreview;
      } else {
        image = null;
      }
    }
  }

  async function pickImage() {
    let res = await selectImage();
    // Handle canceling select image
    if (res === null) return;

    question.image = res.uuid;
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

<div
  tabindex="0"
  role="button"
  class="question__img-wrapper"
  on:click={pickImage}
  on:keypress={onImageKeyPress}
>
  {#if image}
    <img class="question__img" src={image} alt="Uploaded Content" />

    <button class="remove" on:click={removeImage}> Click to remove </button>
  {:else}
    <p>Pick Image</p>
  {/if}
</div>

<!-- Image store access for rendering the image picker -->
<ImageStorage />

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .question__img-wrapper {
    max-height: 50vh;
    width: 100%;
    height: 50vh;
    overflow: hidden;
    position: relative;
    margin-bottom: 1rem;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .remove {
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
  }

  .remove:hover {
    opacity: 1;
  }

  .question__img {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    height: 100%;
    aspect-ratio: auto;
    z-index: -1;
  }
</style>

<!-- Preview image display for a question -->
<script lang="ts">
  import { imagePreviewStore } from "$lib/stores/imageStore";
  import type { ImageFit } from "$lib/api/models";

  // UUID based preview image loading
  export let uuid: string | null = null;
  // Preview image URL for loaded images
  let previewImage: string | null = null;

  // Preloaded question images
  export let preloaded: HTMLImageElement | null = null;

  // Image fitting
  export let fit: ImageFit;

  // Handling for images that need to be loaded from the preview store
  $: if (uuid !== null) {
    // Handle displaying image previews
    let imagePreview = $imagePreviewStore[uuid];
    if (imagePreview !== undefined) {
      previewImage = imagePreview;
    } else {
      previewImage = null;
    }
  }

  /**
   * Handles appending the provided preloaded image HTML
   * element as a child for an image wrapper target element
   *
   * @param target The element to append to
   * @param image The preview image element
   */
  function preloadChild(target: HTMLElement, image: HTMLImageElement) {
    // Prepare the preloaded image HTML element classes
    if (!image.classList.contains("qu-image")) {
      image.classList.add("qu-image");
    }

    // Setup attributes and alt info
    image.setAttribute("data-fit", fit);
    image.alt = "Uploaded Context";

    target.appendChild(image);
  }
</script>

{#if previewImage !== null}
  <div class="qu-image-wrapper">
    <img
      class="qu-image"
      data-fit={fit}
      src={previewImage}
      alt="Uploaded Content"
    />
  </div>
{:else if preloaded != null}
  <div class="qu-image-wrapper" use:preloadChild={preloaded} />
{/if}

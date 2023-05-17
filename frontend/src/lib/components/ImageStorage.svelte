<script lang="ts">
  import { acceptUploadMany } from "$lib/utils/file";
  import { confirmDialog } from "$stores/dialogStore";
  import {
    imageStore,
    selectImageStore,
    uploadFile,
    type StoredImage,
    clearSelectImage,
    consumeSelectImage,
    imagePreviewStore
  } from "$stores/imageStore";
  import Import from "$components/icons/Import.svelte";

  let uploading: FileUpload[] = [];

  interface FileUpload {
    name: string;
    progress: number;
    error: string | null;
  }

  async function doUpload() {
    const files: FileList | null = await acceptUploadMany("image/*");

    // No files were uploaded
    if (files === null) return;

    uploadFiles(files);
  }

  function onDrop(event: DragEvent) {
    event.stopPropagation();
    event.preventDefault();

    const dataTransfer = event.dataTransfer;
    if (dataTransfer === null) return;
    const files = dataTransfer.files;
    if (files === null) return;
    uploadFiles(files);
  }

  function onDragOver(event: DragEvent) {
    event.stopPropagation();
    event.preventDefault();
  }

  function uploadFiles(files: FileList) {
    for (const file of files) {
      uploading.push({
        name: file.name,
        progress: 0,
        error: null
      });

      uploading = uploading;
      uploadFile(file, (progress) => {
        onProgress(file.name, progress);
      })
        .then(() => onUploadComplete(file))
        .catch((error) => onUploadFailed(file, error));
    }
  }

  function onUploadComplete(file: File) {
    uploading = uploading.filter((value) => value.name !== file.name);
  }

  function onUploadFailed(file: File, error: Error) {
    uploading = uploading.map((value) => {
      if (value.name === file.name) {
        value.error = error.message;
      }
      return value;
    });
  }

  function onProgress(name: string, progress: number) {
    uploading = uploading.map((value) => {
      if (value.name === name) {
        value.progress = progress;
      }
      return value;
    });
  }

  function onDelete(image: StoredImage) {
    // Remove the image from the image store
    imageStore.update((store) => {
      return store.filter((value) => value.uuid !== image.uuid);
    });

    // Remove the image preview
    imagePreviewStore.update((store) => {
      delete store[image.uuid];
      return store;
    });
  }

  async function doClear() {
    const confirmed = await confirmDialog(
      "Confirm Deletion",
      "Are you sure you want to delete all the uploaded files?"
    );

    if (!confirmed) return;

    imageStore.set([]);
    imagePreviewStore.set({});
    uploading = [];
  }
</script>

{#if $selectImageStore}
  <div class="wrapper">
    <div class="dialog" on:drop={onDrop} on:dragover={onDragOver}>
      <div class="images">
        <!-- Don't show the upload info if already uploading -->
        {#if $imageStore.length < 1 && uploading.length < 1}
          <p class="images__text">
            Click upload or drag and drop files here to upload
          </p>
        {/if}

        {#each $imageStore as image}
          <div class="file">
            <p class="file__name">{image.name}</p>
            <div class="image-wrapper">
              {#if $imagePreviewStore[image.uuid] !== undefined}
                <img
                  class="image"
                  src={$imagePreviewStore[image.uuid]}
                  alt="Preview"
                />
              {:else}
                <span>Preview loading..</span>
              {/if}
            </div>
            <p>{image.size} bytes</p>
            <div class="file__actions">
              <button
                class="btn btn--small"
                on:click={() => consumeSelectImage(image)}>Select</button
              >
              <button class="btn btn--small" on:click={() => onDelete(image)}
                >Delete</button
              >
            </div>
          </div>
        {/each}

        {#each uploading as upload}
          <div class="file">
            <p>{upload.name}</p>
            {#if upload.error}
              <p class="error">{upload.error}</p>
            {:else}
              <p>Progress: {upload.progress}</p>
            {/if}
          </div>
        {/each}
      </div>

      <div class="actions btn-row btn-row--fill">
        <button on:click={clearSelectImage} class="btn btn--surface"
          >Close</button
        >
        <button on:click={doUpload} class="btn btn--icon btn--surface">
          <Import />
          Upload Images
        </button>
        <button
          on:click={doClear}
          class="btn btn--surface"
          disabled={$imageStore.length === 0}>Delete All Images</button
        >
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  @import "../../assets/scheme.scss";

  .error {
    color: #ff8989;
  }

  .wrapper {
    position: fixed;
    z-index: 1;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;

    display: flex;
    justify-content: center;
    align-items: center;

    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }

  .dialog {
    background-color: $surface;
    padding: 1rem;
    border-radius: 0.5rem;
    max-width: 48rem;
    width: 100%;
    margin: 1rem;
  }

  .images {
    display: grid;

    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.5rem;

    padding: 0.5rem;
    border-radius: 0.25rem;
    background-color: $surfaceLight;

    min-height: 30vh;
    max-height: 60vh;

    overflow: auto;
    margin-bottom: 1rem;
    position: relative;

    &__text {
      position: absolute;
      left: 50%;
      top: 50%;
      max-width: 30%;
      text-align: center;
      transform: translate(-50%, -50%);
    }
  }

  .image {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 100%;

    &-wrapper {
      width: 100%;
      height: 80px;
      overflow: hidden;
      position: relative;
    }
  }

  .file {
    border: 1px solid $surface;
    // background-color: $surface;
    box-shadow: 0 0 5px rgba(0, 0, 0, 0.5);
    padding: 0.5rem;
    border-radius: 0.25rem;

    &__name {
      text-overflow: ellipsis;
      overflow: hidden;
      color: #ffffff;
      font-weight: bold;
      margin-bottom: 0.5rem;
    }

    &__actions {
      display: flex;
      flex-flow: column;
      gap: 0.5rem;
    }
  }

  .actions {
    display: flex;
    gap: 1rem;
  }

  @media screen and (max-width: 48rem) {
    .images {
      grid-template-columns: 1fr 1fr;
    }
  }
  @media screen and (max-width: 32rem) {
    .images {
      grid-template-columns: 1fr;
    }

    .actions {
      display: flex;
      flex-flow: column;
    }
  }
</style>

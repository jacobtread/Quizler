<script lang="ts">
  import { acceptUploadMany } from "$lib/file";
  import {
    imageStore,
    selectImageStore,
    uploadFile,
    type StoredImage,
    clearSelectImage,
    consumeSelectImage,
    imagePreviewStore
  } from "$stores/imageStore";

  let uploading: FileUpload[] = [];
  let dragging = false;

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

    dragging = false;
    const dataTransfer = event.dataTransfer;
    if (dataTransfer === null) return;
    const files = dataTransfer.files;
    if (files === null) return;
    uploadFiles(files);
  }

  function onDragEnter() {
    dragging = true;
  }

  function onDragExit() {
    dragging = false;
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
        .then(() => {
          uploading = uploading.filter((value) => value.name !== file.name);
        })
        .catch((error) => {
          uploading = uploading.map((value) => {
            if (value.name === file.name) {
              value.error = error;
            }
            return value;
          });
        });
    }
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
</script>

{#if $selectImageStore}
  <div class="wrapper">
    <div
      class="dialog"
      on:drop={onDrop}
      on:dragenter={onDragEnter}
      on:dragleave={onDragExit}
      on:dragover={onDragOver}
    >
      <button on:click={clearSelectImage}>Close</button>
      <div class="images">
        {#each $imageStore as image}
          <div class="file">
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
            <p>{image.name}</p>
            <p>{image.size} bytes</p>
            <button on:click={() => consumeSelectImage(image)}>Select</button>
            <button on:click={() => onDelete(image)}>Delete</button>
          </div>
        {/each}

        {#each uploading as upload}
          <div class="file">
            {#if upload.error}
              <p class="error">{upload.error}</p>
            {/if}
            <p>Progress: {upload.progress}</p>
            <p>{upload.name}</p>
          </div>
        {/each}
      </div>

      <button on:click={doUpload}>Upload Images</button>

      <p>Click upload or drag and drop files here to upload</p>
    </div>
  </div>
{/if}

<style>
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
  }

  .dialog {
    background-color: #333;
    padding: 1rem;
    border: 1px solid #999;
    max-width: 48rem;
    width: 100%;
  }

  .images {
    display: grid;

    grid-template-columns: repeat(4, 1fr);
    grid-template-rows: 1fr 1fr;

    gap: 1rem;
    padding: 0.5rem;
    border: 1px solid #777;
    margin: 1rem 0;

    max-height: 60vh;
    overflow: auto;
  }

  .image-wrapper {
    width: 100%;
    height: 80px;
    overflow: hidden;
    position: relative;
  }

  .image {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 100%;
  }
</style>

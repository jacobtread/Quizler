<script lang="ts">
  import { get } from "svelte/store";
  import {
    imageStore,
    selectImageStore,
    uploadFile,
    type StoredImage,
  } from "./imageStore";

  let input: HTMLInputElement;
  let uploading: FileUpload[] = [];

  interface FileUpload {
    name: string;
    progress: number;
    error: string | null;
  }

  async function onUpload() {
    for (let i = 0; i < input.files.length; i++) {
      let file: File = input.files.item(i);
      uploading.push({
        name: file.name,
        progress: 0,
        error: null,
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

  function onClose() {
    selectImageStore.set({ visible: false, callback: null });
  }

  function onSelect(image: StoredImage) {
    let store = get(selectImageStore);
    if (store.callback) {
      store.callback(image);
    }
    onClose();
  }

  function onDelete(image: StoredImage) {
    imageStore.update((store) => {
      return store.filter((value) => value.uuid !== image.uuid);
    });
  }
</script>

{#if $selectImageStore.visible}
  <div class="wrapper">
    <div class="dialog">
      <button on:click={onClose}>Close</button>
      <div class="images">
        {#each $imageStore as image}
          <div class="file">
            <div class="image-wrapper">
              {#if image.previewUrl != null}
                <img class="image" src={image.previewUrl} alt="Preview" />
              {:else}
                <span>Preview loading..</span>
              {/if}
            </div>
            <p>{image.name}</p>
            <p>{image.size} bytes</p>
            <button on:click={() => onSelect(image)}>Select</button>
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
      <input
        hidden
        type="file"
        multiple
        name=""
        id=""
        bind:this={input}
        on:change={onUpload}
      />

      <button on:click={() => input.click()}>Upload Images</button>
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

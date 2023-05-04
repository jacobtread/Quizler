<script lang="ts">
  import { imageStore, uploadFile } from "./imageStore";

  let input: HTMLInputElement;
  let uploading: FileUpload[] = [];

  interface FileUpload {
    name: string;
    progress: number;
  }

  async function onUpload() {
    for (let i = 0; i < input.files.length; i++) {
      let file: File = input.files.item(i);
      uploading.push({
        name: file.name,
        progress: 0,
      });
      uploading = uploading;
      uploadFile(file, (progress) => {
        onProgress(file.name, progress);
      }).then(() => {
        uploading = uploading.filter((value) => value.name !== file.name);
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
</script>

<div>
  <div>
    {#each $imageStore as image}
      <div class="file">
        <div class="image">
          {#if image.previewUrl != null}
            <img src={image.previewUrl} alt="Preview" />
          {:else}
            <span>Preview loading..</span>
          {/if}
        </div>
        <p>{image.name}</p>
        <p>{image.size} bytes</p>
      </div>
    {/each}

    {#each uploading as upload}
      <div class="file">
        <p>Progress: {upload.progress}</p>
        <p>{upload.name}</p>
      </div>
    {/each}
  </div>
  <input type="file" name="" id="" bind:this={input} />

  <button on:click={onUpload}>Upload Images</button>
</div>

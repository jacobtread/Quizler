<script lang="ts">
  import { flip } from "svelte/animate";

  import { ClientMessage, errorText, ServerError } from "$api/models";
  import * as socket from "$api/socket";
  import { createHttp } from "$api/http";
  import * as constants from "$lib/constants";

  import QuestionListItem from "$components/QuestionListItem.svelte";
  import FloatingLoader from "$components/FloatingLoader.svelte";
  import Import from "$components/icons/Import.svelte";
  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";
  import Export from "$components/icons/Export.svelte";
  import Add from "$components/icons/Add.svelte";

  import { loadQuizBlob, createQuizBlob } from "$lib/utils/format";
  import { acceptUpload, startDownload } from "$lib/utils/file";

  import { setHome, setGame } from "$stores/state";
  import { imageStore, type StoredImage } from "$stores/imageStore";
  import { errorDialog } from "$stores/dialogStore";
  import {
    createData,
    shuffleQuestions,
    type CreateData,
    addQuestion
  } from "$stores/createStore";
  import { tweened, type Tweened } from "svelte/motion";
  import Cog from "$lib/components/icons/Cog.svelte";
  import Settings from "$lib/components/editor/Settings.svelte";

  let loading: boolean = false;
  let loadingState: string = "";
  let progress: Tweened<number> = tweened(0);

  async function doExport() {
    const data: CreateData = $createData;
    const images: StoredImage[] = $imageStore;

    console.debug("Exporting quiz to file", data.name);

    // Create a blob from the quiz contents
    const blob = await createQuizBlob(data, images);

    // Start the file download
    const fileName = data.name + ".quizler";
    startDownload(fileName, blob);
  }

  async function doImport() {
    const file: File | null = await acceptUpload(".quizler");

    // No file was uploaded
    if (file === null) return;

    try {
      const imported: CreateData = await loadQuizBlob(file);

      // Update the store
      createData.set(imported);

      console.debug("Imported quiz file", imported);
    } catch (e) {
      console.error("Error while importing quiz file", e);
      errorDialog("Failed to import", "Quiz file invalid or corrupted");
    }
  }

  function onUploadProgress(event: ProgressEvent) {
    if (event.lengthComputable) {
      const percentComplete = (event.loaded / event.total) * 100;
      console.debug(`Uploading content: ${percentComplete.toFixed(0)}%`);
      progress.set(percentComplete);
    }
  }

  function play() {
    loading = true;
    loadingState = "Uploading";

    const data: CreateData = $createData;

    // Trim name whitespace
    data.name = data.name.trim();
    // Trim text whitespace
    data.text = data.text.trim();

    const images: StoredImage[] = $imageStore;

    console.debug("Creating quiz");

    // Send the creation request to the HTTP API
    createHttp(data, images, onUploadProgress)
      // Initialize the created game
      .then((uuid) => {
        loadingState = "Initializing";
        console.debug("Initializing game", uuid);

        return socket.send({ ty: ClientMessage.Initialize, uuid });
      })
      // Switch to the game view
      .then(({ id, token, config }) => {
        setGame({ id, token, config, host: true });
      })
      // Handle errors
      .catch((error: Error | ServerError) => {
        console.error("Failed to create", error);
        if (error instanceof Error) {
          errorDialog("Failed to create", error.message);
        } else {
          errorDialog("Failed to create", errorText[error]);
        }
      })
      .finally(() => (loading = false));
  }

  let settings: boolean = false;
</script>

{#if loading}
  {#if loadingState === "Uploading"}
    <FloatingLoader text={`Uploading ${$progress.toFixed(0)}%`} />
  {:else}
    <FloatingLoader text="Connecting..." />
  {/if}
{/if}

{#if settings}
  <Settings bind:visible={settings} />
{/if}

<main class="main">
  <aside class="sidebar">
    <button on:click={setHome} class="btn btn--icon btn--l">
      <Back />
      <span>Back</span>
    </button>
    <button on:click={doImport} class="btn btn--icon btn--l">
      <Import />
      <span>Import</span>
    </button>
    <button on:click={doExport} class="btn btn--icon btn--l">
      <Export />
      <span>Export</span>
    </button>
    <button on:click={() => (settings = true)} class="btn btn--icon btn--l">
      <Cog />
      <span>Settings</span>
    </button>
    <button on:click={play} class="btn btn--icon btn--l">
      <Play />
      <span>Play</span>
    </button>
    <button
      on:click={shuffleQuestions}
      disabled={$createData.questions.length <= 1}
      class="btn btn--l"
    >
      Shuffle
    </button>
    <button
      on:click={addQuestion}
      disabled={$createData.questions.length >= constants.MAX_QUESTIONS}
      class="btn btn--icon btn--l"
    >
      <Add />
      Add Question
    </button>
  </aside>

  <div class="list">
    <div class="list__content">
      <ol class="questions">
        {#each $createData.questions as question, index (question.id)}
          <li animate:flip={{ duration: 500 }}>
            <QuestionListItem
              bind:question
              {index}
              length={$createData.questions.length}
            />
          </li>
        {/each}
      </ol>
    </div>
  </div>
</main>

<style lang="scss">
  @import "../../assets/scheme.scss";

  .main {
    height: 100%;
    width: 100%;

    display: flex;
    flex-flow: row;
    padding: 1rem;
    gap: 1rem;
  }

  .sidebar {
    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  .list {
    flex: auto;
    overflow: auto;
  }

  .questions {
    display: flex;
    gap: 1rem;
    flex-flow: column;
    list-style: none;
  }

  .header {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    padding-bottom: 1rem;

    .btn {
      flex: auto;
      text-align: center;
      justify-content: center;
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

  @media screen and (max-width: 86rem) {
    .main {
      grid-template-columns: 1fr 1fr;
    }
  }

  @media screen and (max-width: 64rem) {
    .main {
      grid-template-columns: 1fr;
      grid-template-rows: 1fr;
      gap: 0;
      overflow: auto;
    }

    .list {
      overflow: visible;

      &__actions {
        position: sticky;
        top: 0;
        left: 0;
        z-index: 1;
      }

      &__content {
        padding: 1rem;
      }
    }

    .details {
      overflow: visible;
      max-width: unset;
      padding: 1rem;
    }

    .header,
    .list__actions {
      flex-wrap: wrap;
    }
  }
</style>

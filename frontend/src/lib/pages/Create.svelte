<script lang="ts">
  import {
    ClientMessage,
    errorText,
    ServerError,
    QuestionType,
    createDataSchema,
    type CreateData,
    type CreateDataRuntime
  } from "$api/models";
  import * as socket from "$api/socket";
  import { createHttp } from "$api/http";

  import FloatingLoader from "$components/FloatingLoader.svelte";
  import Import from "$components/icons/Import.svelte";
  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";
  import Export from "$components/icons/Export.svelte";

  import { loadQuizBlob, createQuizBlob } from "$lib/utils/format";
  import { acceptUpload, startDownload } from "$lib/utils/file";

  import { setHome, setGame } from "$stores/state";
  import { imageStore, type StoredImage } from "$stores/imageStore";
  import { errorDialog } from "$stores/dialogStore";
  import {
    createData,
    setCreateData,
    activeQuestion
  } from "$stores/createStore";
  import { tweened, type Tweened } from "svelte/motion";
  import Cog from "$lib/components/icons/Cog.svelte";
  import Settings from "$lib/components/editor/Settings.svelte";
  import QuestionEditor from "../components/editor/QuestionEditor.svelte";
  import QuestionList from "$lib/components/editor/QuestionList.svelte";

  let loading: boolean = false;
  let loadingState: string = "";
  let progress: Tweened<number> = tweened(0);
  let settings: boolean = false;

  async function doExport() {
    const data: CreateData | null = getCreateData();
    if (data === null) return;
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
      const imported: CreateDataRuntime = await loadQuizBlob(file);

      // Update the store
      setCreateData(imported);

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

  /**
   * Creates a copy of the current create data returns a
   * validated copy or null if validation failed
   */
  function getCreateData(): CreateData | null {
    let output = createDataSchema.safeParse($createData);

    if (!output.success) {
      let errors = "";
      let currentIndex = -1;

      output.error.issues.forEach((issue) => {
        const message = issue.message;
        const path = issue.path;

        let leading = "";

        if (path.length > 1 && path[0] === "questions") {
          const questionIndex = path[1] as number;

          if (questionIndex != currentIndex) {
            errors += "Question " + (questionIndex + 1) + ":\n";
            currentIndex = questionIndex;
          }

          if (path.length > 3 && path[2] === "answers") {
            const questionIndex = path[3] as number;
            leading += "* Answer " + (questionIndex + 1) + " ";
          }
        }

        errors += `${leading} ${message} \n`;
      });

      errorDialog(`Quiz has error(s)`, errors);

      return null;
    }

    // Ensure multiple choice questions have valid correct_answers field
    for (const question of output.data.questions) {
      // Ensure the correct_answers field is correct
      if (question.ty === QuestionType.Multiple) {
        let correct = 0;
        for (const answer of question.answers) {
          if (answer.correct) correct++;
        }
        question.correct_answers = correct;
      }
    }

    return output.data;
  }

  function play() {
    const data: CreateData | null = getCreateData();

    if (data === null) return;

    loading = true;
    loadingState = "Uploading";

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
</script>

{#if loading}
  {#if loadingState === "Uploading"}
    <FloatingLoader text={`Uploading ${$progress.toFixed(0)}%`} />
  {:else}
    <FloatingLoader text="Connecting..." />
  {/if}
{/if}

<Settings bind:visible={settings} />

<main class="main">
  <header class="header btn-row btn-row--fill">
    <button on:click={setHome} class="btn btn--icon">
      <Back />
      <span>Back</span>
    </button>
    <button on:click={doImport} class="btn btn--icon">
      <Import />
      <span>Import</span>
    </button>
    <button on:click={doExport} class="btn btn--icon">
      <Export />
      <span>Export</span>
    </button>
    <button on:click={() => (settings = true)} class="btn btn--icon">
      <Cog />
      <span>Quiz Settings</span>
    </button>
    <button on:click={play} class="btn btn--icon">
      <Play />
      <span>Play</span>
    </button>
  </header>

  <div class="wrapper">
    <QuestionList />
    <div class="editor">
      {#if $activeQuestion !== null}
        <QuestionEditor bind:question={$activeQuestion} />
      {:else}
        <div class="editor__none">
          <h1>Select a question</h1>
          <p>No question selected. Select a question to start editing</p>
        </div>
      {/if}
    </div>
  </div>
</main>

<style lang="scss">
  @import "../../assets/scheme.scss";

  .editor__none {
    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;
    flex: auto;

    h1 {
      color: #fff;
    }
  }

  .main {
    height: 100%;
    width: 100%;

    display: flex;
    flex-flow: column;
    padding: 1rem;
    gap: 1rem;
    overflow: hidden;
  }

  .editor {
    flex: auto;
    overflow: auto;
    display: flex;
    flex-flow: column;
  }

  .wrapper {
    flex: auto;
    display: flex;
    overflow: hidden;
    gap: 1rem;
  }

  .header {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  @media screen and (max-width: 64rem) {
    .wrapper {
      flex-flow: column-reverse;
    }
  }

  @media screen and (max-width: 64rem), (max-height: 48rem) {
    .editor {
      display: block;
    }
  }
</style>

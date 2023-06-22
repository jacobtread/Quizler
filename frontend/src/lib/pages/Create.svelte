<script lang="ts">
  import { flip } from "svelte/animate";

  import {
    dndzone,
    type DndEvent,
    SHADOW_ITEM_MARKER_PROPERTY_NAME
  } from "svelte-dnd-action";

  import {
    ClientMessage,
    errorText,
    ServerError,
    type Question,
    QuestionType
  } from "$api/models";
  import * as socket from "$api/socket";
  import { createHttp } from "$api/http";
  import * as constants from "$lib/constants";

  import QuestionListItem from "$lib/components/editor/QuestionListItem.svelte";
  import FloatingLoader from "$components/FloatingLoader.svelte";
  import Import from "$components/icons/Import.svelte";
  import Back from "$components/icons/Back.svelte";
  import Play from "$components/icons/Play.svelte";
  import Export from "$components/icons/Export.svelte";
  import Add from "$components/icons/Add.svelte";
  import Shuffle from "$components/icons/Shuffle.svelte";

  import { loadQuizBlob, createQuizBlob } from "$lib/utils/format";
  import { acceptUpload, startDownload } from "$lib/utils/file";

  import { setHome, setGame } from "$stores/state";
  import { imageStore, type StoredImage } from "$stores/imageStore";
  import { errorDialog } from "$stores/dialogStore";
  import {
    createData,
    shuffleQuestions,
    type CreateData,
    addQuestion,
    setCreateData,
    activeQuestion
  } from "$stores/createStore";
  import { tweened, type Tweened } from "svelte/motion";
  import Cog from "$lib/components/icons/Cog.svelte";
  import Settings from "$lib/components/editor/Settings.svelte";
  import QuestionEditor from "../components/editor/QuestionEditor.svelte";

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

  function validate(data: CreateData): boolean {
    // TODO: Visual validation failure hints instead of dialog
    for (let i = 0; i < data.questions.length; i++) {
      const question = data.questions[i];

      // Trim whitespace from the text
      question.text = question.text.trim();

      if (question.text.length < 1) {
        errorDialog(
          "Empty quesiton",
          `Question ${i + 1} text must not be empty`
        );
        return false;
      }

      if (
        question.ty === QuestionType.Single ||
        question.ty === QuestionType.Multiple
      ) {
        for (let j = 0; j < question.answers.length; j++) {
          const answer = question.answers[j];
          answer.value = answer.value.trim();

          if (answer.value.length < 1) {
            errorDialog(
              "Empty answer",
              `Answer number ${j + 1} of question ${i + 1} must not be blank`
            );
            return false;
          }
        }
      } else if (question.ty === QuestionType.Typer) {
        // Trim answers
        for (let j = 0; j < question.answers.length; j++) {
          const trimmed = question.answers[j].trim();
          question.answers[j] = trimmed;
          if (trimmed.length < 1) {
            errorDialog(
              "Empty answer",
              `Answer number ${j + 1} of question ${i + 1} must not be blank`
            );
            return false;
          }
        }
      }

      // Ensure the correct_answers field is correct
      if (question.ty === QuestionType.Multiple) {
        let correct = 0;
        for (const answer of question.answers) {
          if (answer.correct) correct++;
        }
        question.correct_answers = correct;
      }
    }

    return true;
  }

  function play() {
    const data: CreateData = $createData;

    if (!validate(data)) return;

    loading = true;
    loadingState = "Uploading";

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

  function handleDndConsider(e: CustomEvent<DndEvent<Question>>) {
    $createData.questions = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent<DndEvent<Question>>) {
    $createData.questions = e.detail.items;
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
    <div class="list">
      <button
        on:click={shuffleQuestions}
        disabled={$createData.questions.length <= 1}
        class="btn btn--icon"
      >
        <Shuffle />
        Shuffle
      </button>
      <section
        class="questions"
        use:dndzone={{
          items: $createData.questions,
          flipDurationMs: 200,
          dropTargetStyle: {}
        }}
        on:consider={handleDndConsider}
        on:finalize={handleDndFinalize}
      >
        {#each $createData.questions as question, index (question.id)}
          <div style="position: relative;" animate:flip={{ duration: 200 }}>
            <QuestionListItem {question} {index} />
            {#if question[SHADOW_ITEM_MARKER_PROPERTY_NAME]}
              <div class="shadow-item" />
            {/if}
          </div>
        {/each}
      </section>
      <button
        on:click={addQuestion}
        disabled={$createData.questions.length >= constants.MAX_QUESTIONS}
        class="btn add btn--icon-only"
      >
        <Add />
      </button>
    </div>
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
  .shadow-item {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    visibility: visible;
    border: 3px dashed #333;
    border-radius: 0.25rem;
    margin: 0;
  }

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

  .list {
    display: flex;
    flex-flow: column;
    gap: 1rem;

    min-width: 14rem;
  }

  .questions {
    position: relative;
    padding: 1rem;
    overflow: auto;
    flex: auto;
    border: 0.1rem solid $surface;
    border-radius: 0.25rem;

    display: flex;
    gap: 1rem;
    flex-flow: column;
    list-style: none;
  }

  .header {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  @media screen and (max-width: 64rem) {
    .list {
      flex-flow: row;
      width: auto;
    }

    .wrapper {
      flex-flow: column-reverse;
    }

    .questions {
      flex-flow: row;
    }
  }

  @media screen and (max-width: 64rem), (max-height: 48rem) {
    .editor {
      display: block;
    }
  }

  @media screen and (max-width: 48rem), (max-height: 48rem) {
    .questions {
      padding: 0;
      align-items: center;
    }
  }
</style>

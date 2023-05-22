<script lang="ts">
  import { flip } from "svelte/animate";

  import {
    ClientMessage,
    errorText,
    ServerError,
    NameFiltering
  } from "$api/models";
  import * as socket from "$api/socket";
  import { createHttp } from "$api/http";
  import * as constants from "$lib/constants";

  import QuestionListItem from "$components/QuestionListItem.svelte";
  import TimeInput from "$components/TimeInput.svelte";
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
      console.debug(`Uploading content: ${percentComplete}%`);
    }
  }

  function play() {
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
      .then((uuid) => socket.send({ ty: ClientMessage.Initialize, uuid }))
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
      });
  }
</script>

<main class="main">
  <div class="details">
    <header class="header">
      <button on:click={setHome} class="btn btn--icon">
        <Back />
        Back
      </button>
      <button on:click={doImport} class="btn btn--icon">
        <Import />
        Import
      </button>
      <button on:click={doExport} class="btn btn--icon">
        <Export />
        Export
      </button>
      <button on:click={play} class="btn btn--icon">
        <Play />
        Play
      </button>
    </header>
    <label class="field">
      <span class="field__name">Title</span>
      <p class="field__desc">
        Give your quiz a title <span class="optional">Optional</span>
      </p>
      <input class="input" type="text" bind:value={$createData.name} />
    </label>
    <label class="field">
      <span class="field__name">Description</span>
      <p class="field__desc">
        Description of your Quiz <span class="optional">Optional</span>
      </p>
      <textarea
        class="input input--desc"
        name=""
        id=""
        cols="30"
        rows="5"
        bind:value={$createData.text}
      />
    </label>
    <div class="field">
      <span class="field__name">Wait Time</span>
      <p class="field__desc">Time to wait between each question</p>
      <TimeInput
        bind:value={$createData.timing.wait_time}
        min={constants.MIN_WAIT_TIME}
        max={constants.MAX_WAIT_TIME}
      />
    </div>
    <label class="field">
      <span class="field__name">Max Players</span>
      <p class="field__desc">
        Maximum number of players allowed to join this quiz
      </p>
      <input
        class="input"
        type="number"
        bind:value={$createData.max_players}
        min={constants.MIN_MAX_PLAYERS}
        max={constants.MAX_MAX_PLAYERS}
      />
    </label>
    <label class="field">
      <span class="field__name">Name Filtering</span>
      <p class="field__desc">
        Level of filtering on profane/inappropriate naming. Its recommended that
        you leave this on Medium or High
      </p>
      <select bind:value={$createData.filtering} class="input">
        <option value={NameFiltering.None}>None: Don't filter names</option>
        <option value={NameFiltering.Low}
          >Low: Filter out more severe names</option
        >
        <option value={NameFiltering.Medium}>
          Medium: Filter out anything thats not mild
        </option>
        <option value={NameFiltering.High}>
          High: Filter out as much as possible
        </option>
      </select>
    </label>
  </div>

  <div class="list">
    <div class="list__actions">
      <button
        on:click={addQuestion}
        disabled={$createData.questions.length >= constants.MAX_QUESTIONS}
        class="btn btn--icon"
      >
        <Add />
        Add Question
      </button>
      <button
        on:click={shuffleQuestions}
        disabled={$createData.questions.length <= 1}
        class="btn btn--sm"
      >
        Shuffle
      </button>
    </div>

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

  .optional {
    color: #777;
    margin-left: 0.5rem;
  }

  .main {
    height: 100%;
    width: 100%;

    display: grid;
    grid-template-columns: 1fr 1.75fr;
    grid-template-rows: 100%;
  }

  .details {
    overflow: auto;
    height: 100%;
    padding: 1rem 0 0 1rem;
  }

  .list {
    height: 100%;
    display: flex;
    flex-flow: column;

    &__actions {
      background-color: $appBackground;
      padding: 1rem;
      display: flex;
      gap: 1rem;

      .btn {
        flex: auto;
        text-align: center;
        justify-content: center;
      }
    }

    &__content {
      flex: auto;
      overflow: auto;
      padding: 0 1rem 1rem;
    }
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

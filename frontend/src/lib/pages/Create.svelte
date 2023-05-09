<script lang="ts">
  import {
    ClientMessage,
    type CreatedResponse,
    type Question,
    type TimingConfig,
    type CreateRequest,
    errorText,
    ServerError
  } from "$lib/socket/models";
  import * as socket from "$lib/socket";
  import {
    DEBUG,
    MAX_WAIT_TIME,
    MIN_WAIT_TIME,
    defaultQuestion
  } from "$lib/constants";
  import Back from "$lib/assets/icons/back.svg";
  import Import from "$lib/assets/icons/import.svg";
  import Export from "$lib/assets/icons/export.svg";
  import Play from "$lib/assets/icons/play.svg";
  import QuestionEditor from "$components/QuestionEditor.svelte";
  import QuestionList from "$components/QuestionList.svelte";
  import ImageStorage from "$components/ImageStorage.svelte";
  import { get } from "svelte/store";
  import { imageStore } from "$stores/imageStore";
  import { loadQuizBlob, createQuizBlob } from "$lib/format";
  import { setGame, setHome } from "$stores/state";
  import TimeInput from "$components/TimeInput.svelte";
  import { confirmDialog, errorDialog } from "$lib/stores/dialogStore";
  import { acceptUpload, startDownload } from "$lib/file";

  // Questions array
  let questions: Question[] = [defaultQuestion()];

  // Active question being edited
  let editing: Question | null = null;

  // Name of the quiz
  let name: string = "Example Quiz";

  // Quiz description text
  let text: string = "Small description about your quiz";

  // Game timing configuration
  let timing: TimingConfig = {
    wait_time: 1000 * 10
  };

  async function back() {
    const result = await confirmDialog(
      "Confirm Back",
      "Are you sure you want to go back? You will loose any unsaved progress"
    );

    if (result) {
      setHome();
    }
  }

  async function doExport() {
    console.debug("Exporting quiz to file", name);

    // Create a blob from the quiz contents
    const blob = await createQuizBlob(name, text, timing, questions);

    // Start the file download
    const fileName = name + ".quizler";
    startDownload(fileName, blob);
  }

  async function doImport() {
    const file: File | null = await acceptUpload(".quizler");

    // No file was uploaded
    if (file === null) return;

    try {
      const imported = await loadQuizBlob(file);
      questions = imported.questions;
      name = imported.name;
      text = imported.text;
      timing = imported.timing;

      console.debug("Imported quiz file", imported);
    } catch (e) {
      console.error("Error while importing quiz file", e);
      errorDialog("Failed to import", "Quiz file invalid or corrupted");
    }
  }

  /**
   * Uses the HTTP API to create the Quiz returning the
   * UUID of the quiz that was prepared for initialization
   *
   * @param config The quiz config
   */
  async function createHttp(config: CreateRequest): Promise<string> {
    // Create the form to upload
    const form = new FormData();
    // Append the config
    form.append("config", JSON.stringify(config));

    // Load the images from the image store
    const images = get(imageStore);
    // Append the images to the form
    images.forEach((image) => form.append(image.uuid, image.blob));

    const request: XMLHttpRequest = new XMLHttpRequest();

    // Listen to the upload progress
    request.upload.onprogress = onUploadProgress;
    // Accept JSON responses
    request.responseType = "json";

    // Await failure or response from request (TODO: Handle this reject case)
    await new Promise((resolve, reject) => {
      // Handle success
      request.onload = resolve;
      // Handle all failure callbacks
      request.onerror = request.ontimeout = request.onabort = reject;

      // Create the URL to the create endpoint
      const url = new URL(
        "/api/quiz",
        DEBUG ? "http://localhost" : window.location.origin
      );

      // Set the request method and URL
      request.open("POST", url);

      // Send the multipart form body
      request.send(form);
    });

    const response: CreatedResponse = request.response;
    return response.uuid;
  }

  function onUploadProgress(event: ProgressEvent) {
    if (event.lengthComputable) {
      const percentComplete = (event.loaded / event.total) * 100;
      console.debug(`Uploading content: ${percentComplete}%`);
    }
  }

  async function doPlay() {
    console.debug("Creating quiz");

    const config: CreateRequest = {
      name,
      text,
      timing,
      questions
    };

    const uuid = await createHttp(config);

    console.debug("Quiz waiting for initialize", uuid);

    // Await the socket being alive
    await socket.ready();

    try {
      const { id, token, config } = await socket.send({
        ty: ClientMessage.Initialize,
        uuid
      });

      setGame({ id, token, config, host: true });
    } catch (e) {
      const error = e as ServerError;
      console.error("Failed to initialize", error);
      errorDialog("Failed to create", errorText[error]);
    }
  }
</script>

{#if !editing}
  <header class="header">
    <button on:click={back} class="icon-button">
      <img src={Back} alt="Back" class="icon-button__img" />
      <span class="icon-button__text">Back</span>
    </button>
    <button on:click={doImport} class="icon-button">
      <img src={Import} alt="Import" class="icon-button__img" />
      <span class="icon-button__text">Import</span>
    </button>
    <button on:click={doExport} class="icon-button">
      <img src={Export} alt="Export" class="icon-button__img" />
      <span class="icon-button__text">Export</span>
    </button>
    <button on:click={doPlay} class="icon-button">
      <img src={Play} alt="Play" class="icon-button__img" />
      <span class="icon-button__text">Play</span>
    </button>
    <h1>Create Quiz</h1>
  </header>
{/if}

<main class="main">
  {#if editing}
    <QuestionEditor question={editing} back={() => (editing = null)} />
  {:else}
    <div>
      <label class="field">
        <span class="field__name">Title</span>
        <p class="field__desc">Give your quiz a title</p>
        <input class="input" type="text" bind:value={name} />
      </label>
      <label class="field">
        <span class="field__name">Description</span>
        <p class="field__desc">Describe your quiz</p>
        <textarea
          class="input input--desc"
          name=""
          id=""
          cols="30"
          rows="10"
          bind:value={text}
        />
      </label>
    </div>

    <div class="field">
      <span class="field__name">Wait Time</span>
      <p class="field__desc">Time to wait between each question</p>
      <TimeInput
        bind:value={timing.wait_time}
        min={MIN_WAIT_TIME}
        max={MAX_WAIT_TIME}
      />
    </div>

    <QuestionList {questions} bind:editing />
  {/if}
</main>

<ImageStorage />

<style lang="scss">
  @import "../assets/scheme.scss";

  .main {
    height: 100%;
    padding: 1rem;
    overflow: auto;
    padding-top: 0;
  }

  .header {
    position: sticky;
    top: 0;
    left: 0;
    display: flex;
    gap: 1rem;
    background-color: $appBackground;
    padding: 1rem;
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

  .title,
  .description {
    display: block;
    margin-bottom: 1rem;
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

  .input--desc {
    resize: vertical;
  }
</style>

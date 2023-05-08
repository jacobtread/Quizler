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
      "Are you sure you want to go back? You will loose any unsave progress"
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

  async function doPlay() {
    const config: CreateRequest = {
      name,
      text,
      timing,
      questions
    };

    console.debug("Creating quiz");

    // Create the form
    const form = new FormData();
    // Append the config
    form.append("config", JSON.stringify(config));

    // Append the images to the form
    const images = get(imageStore);
    for (const image of images) {
      form.append(image.uuid, image.blob);
    }

    const url = new URL(
      "/api/quiz",
      DEBUG ? "http://localhost" : window.location.origin
    );

    let response = await fetch(url, { method: "POST", body: form });
    let json: CreatedResponse = await response.json();

    console.debug("Quiz waiting for initialize", json.uuid);

    console.debug("Waiting for socket ready");

    // Await the socket being alive
    await socket.ready();

    console.debug("Sending initialize message");

    try {
      const { id, token, config } = await socket.send({
        ty: ClientMessage.Initialize,
        uuid: json.uuid
      });

      setGame({ id, token, config, host: true });
    } catch (e) {
      const error = e as ServerError;
      console.error("Failed to initialize", error);
      errorDialog("Failed to create", errorText[error]);
    }
  }
</script>

<main class="main">
  {#if editing}
    <QuestionEditor question={editing} back={() => (editing = null)} />
  {:else}
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

    <div>
      <input type="text" bind:value={name} />
      <textarea name="" id="" cols="30" rows="10" bind:value={text} />
    </div>

    <label for="">
      <span>Wait Time</span>
      <p>Time to wait between each question</p>
      <TimeInput
        bind:value={timing.wait_time}
        min={MIN_WAIT_TIME}
        max={MAX_WAIT_TIME}
      />
    </label>

    <QuestionList {questions} bind:editing />
  {/if}
</main>

<ImageStorage />

<style lang="scss">
  @import "../assets/scheme.scss";

  .main {
    height: 100%;
    padding: 1rem;
  }

  .header {
    display: flex;
    gap: 1rem;
  }
</style>

<script lang="ts">
  import { get } from "svelte/store";
  import { flip } from "svelte/animate";

  import {
    ClientMessage,
    type CreatedResponse,
    errorText,
    ServerError,
    NameFiltering
  } from "$lib/socket/models";
  import * as socket from "$lib/socket";
  import {
    DEBUG,
    MAX_MAX_PLAYERS,
    MAX_WAIT_TIME,
    MIN_MAX_PLAYERS,
    MIN_WAIT_TIME
  } from "$lib/constants";

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
  import { imageStore } from "$stores/imageStore";
  import { errorDialog } from "$stores/dialogStore";
  import {
    createData,
    shuffleQuestions,
    type CreateData,
    addQuestion
  } from "$stores/createStore";

  async function doExport() {
    const data: CreateData = get(createData);

    console.debug("Exporting quiz to file", data.name);

    // Create a blob from the quiz contents
    const blob = await createQuizBlob(
      data.name,
      data.text,
      data.max_players,
      data.filtering,
      data.timing,
      data.questions
    );

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

  /**
   * Uses the HTTP API to create the Quiz returning the
   * UUID of the quiz that was prepared for initialization
   *
   * @param config The quiz config
   */
  async function createHttp(config: CreateData): Promise<string> {
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

    // Await failure or response from request
    await new Promise((resolve, reject) => {
      // Handle success
      request.onload = resolve;

      // Handle all failure callbacks
      request.onerror =
        request.ontimeout =
        request.onabort =
          () => reject(new Error("Failed to connect"));

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

    const statusType = Math.floor(request.status / 100);
    if (statusType !== 2) {
      console.error("Failed invalid request", request.response);

      throw new Error(
        "Invalid request try reloading the page or updating Quizler"
      );
    }

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
    const data: CreateData = $createData;

    console.debug("Creating quiz");

    let uuid: string;
    try {
      uuid = await createHttp(data);
    } catch (e) {
      const error = e as Error;
      errorDialog("Failed to create", error.message);
      return;
    }

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

  function back() {
    setHome();
  }
</script>

<main class="main">
  <div class="details">
    <header class="header">
      <button on:click={back} class="btn btn--icon">
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
      <button on:click={doPlay} class="btn btn--icon">
        <Play />
        Play
      </button>
    </header>
    <label class="field">
      <span class="field__name">Title</span>
      <p class="field__desc">Give your quiz a title</p>
      <input class="input" type="text" bind:value={$createData.name} />
    </label>
    <label class="field">
      <span class="field__name">Description</span>
      <p class="field__desc">Describe your quiz</p>
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
        min={MIN_WAIT_TIME}
        max={MAX_WAIT_TIME}
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
        min={MIN_MAX_PLAYERS}
        max={MAX_MAX_PLAYERS}
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
        disabled={$createData.questions.length >= 50}
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
  }

  .list__actions {
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

  .list__content {
    flex: auto;
    overflow: auto;
    padding: 0 1rem 1rem;
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
    }

    .details {
      overflow: visible;
      max-width: unset;
      padding: 1rem;
    }

    .list__actions {
      position: sticky;
      top: 0;
      left: 0;
      z-index: 1;
    }

    .list__content {
      padding: 1rem;
    }

    .header,
    .list__actions {
      flex-wrap: wrap;
    }
  }
</style>

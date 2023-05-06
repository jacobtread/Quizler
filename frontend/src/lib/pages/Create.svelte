<script lang="ts">
  import {
    ClientMessage,
    ServerMessage,
    type CreatedResponse,
    type Question,
    type TimingConfig,
    type UploadConfig
  } from "$lib/socket/models";
  import * as socket from "$lib/socket";
  import {
    DEBUG,
    MAX_WAIT_TIME,
    MIN_WAIT_TIME,
    defaultQuestion
  } from "$lib/constants";
  import QuestionEditor from "$components/QuestionEditor.svelte";
  import QuestionList from "$components/QuestionList.svelte";
  import ImageStorage from "$components/ImageStorage.svelte";
  import { get } from "svelte/store";
  import { imageStore } from "$stores/imageStore";
  import { loadQuiz, saveQuiz } from "$lib/format";
  import { setGame, setHome } from "$stores/state";
  import TimeInput from "$components/TimeInput.svelte";
  import { ZodError } from "zod";

  // Input used for loading quiz files
  let loadInput: HTMLInputElement;

  // Questions array
  let questions: Question[] = [defaultQuestion()];

  // Active question being edited
  let editing: Question | null = null;

  // Name of the quiz
  let name: string = "Example Quiz";

  // Quiz description text
  let text: string = "Small description about your quiz";

  let timing: TimingConfig = {
    wait_time: 1000 * 10
  };

  async function startQuiz() {
    const config: UploadConfig = {
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

    const resp = await socket.send(ClientMessage.Initialize, {
      uuid: json.uuid
    });

    if (resp.ty === ServerMessage.Error) {
      console.error("Error while initializing", resp.error);
    } else {
      const { id, token, config } = resp;
      setGame({ id, token, config, host: true });
    }
  }

  /**
   * Handles saving the current quiz to a file
   */
  async function save() {
    console.debug("Saving quiz to file", name);
    await saveQuiz(name, text, timing, questions);
    console.debug("Saved quiz to file");
  }

  /**
   * Handles loading a quiz file when the quiz
   * load file input changes its value
   */
  async function onLoadQuiz() {
    if (loadInput.files == null) {
      console.error("Failed to load quiz, load input missing files");
      return;
    }

    console.debug("Loading quiz file");

    const file: File | null = loadInput.files.item(0);

    if (file == null) {
      console.error("Failed to load quiz, file was null");
      return;
    }

    try {
      const loaded = await loadQuiz(file);

      console.debug("Loaded quiz file", loaded);

      questions = loaded.questions;
      name = loaded.name;
      text = loaded.text;
      timing = loaded.timing;
    } catch (e) {
      if (e instanceof ZodError) {
        // TODO: Display loading failed message
        console.error("Failed to parse quiz file", e);
      } else {
        console.error("Error while loading quiz file", e);
      }
    }
  }

  function back() {
    if (
      confirm(
        "Are you sure you want to go back? You will loose quiz content you've created"
      )
    ) {
      setHome();
    }
  }
</script>

{#if editing}
  <QuestionEditor question={editing} back={() => (editing = null)} />
{:else}
  <button on:click={back}>Back</button>

  <input hidden bind:this={loadInput} type="file" on:change={onLoadQuiz} />

  <h1>Create Quiz</h1>
  <div>
    <button on:click={save}>Save</button>
    <button on:click={() => loadInput.click()}>Load</button>
    <button on:click={startQuiz}>Play</button>
  </div>

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

<ImageStorage />

<style>
</style>

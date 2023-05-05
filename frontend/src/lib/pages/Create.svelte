<script lang="ts">
  import {
    ClientMessageType,
    ServerMessage,
    type CreatedResponse,
    type Question,
    type TimingConfig,
    type UploadConfig
  } from "$lib/socket/models";
  import { getSocketReady, sendMessage } from "$lib/socket";
  import { DEBUG, defaultQuestion } from "$lib/constants";
  import QuestionEditor from "$components/QuestionEditor.svelte";
  import QuestionList from "$components/QuestionList.svelte";
  import ImageStorage from "$components/ImageStorage.svelte";
  import { get } from "svelte/store";
  import { imageStore } from "$stores/imageStore";
  import { loadQuiz, saveQuiz } from "$lib/format";
  import { setGame, setHome } from "$stores/state";

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

  // TODO: Implement fields for changing these timings and question timings
  let timing: TimingConfig = {
    bonus_score_time: 1000,
    wait_time: 1000
  };

  async function startQuiz() {
    const config: UploadConfig = {
      basic: { name, text },
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
    await getSocketReady();

    console.debug("Sending initialize message");

    const resp = await sendMessage({
      ty: ClientMessageType.Initialize,
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

    const loaded = await loadQuiz(file);

    console.debug("Loaded quiz file", loaded);

    questions = loaded.questions;
    name = loaded.name;
    text = loaded.text;
    timing = loaded.timing;
  }
</script>

{#if editing}
  <QuestionEditor question={editing} back={() => (editing = null)} />
{:else}
  <button on:click={setHome}>Back</button>

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

  <QuestionList {questions} bind:editing />
{/if}

<ImageStorage />

<style>
</style>

<script lang="ts">
  import {
    ClientMessageType,
    type CreatedResponse,
    type GameConfig,
    type Question,
    type TimingConfig,
    type UploadConfig,
  } from "./socket/models";
  import QuestionEditor from "./QuestionEditor.svelte";
  import { DEBUG, defaultQuestion } from "./constants";
  import QuestionList from "./QuestionList.svelte";
  import { getSocketReady, sendMessage, socketReady } from "./socket";
  import { get } from "svelte/store";
  import { imageStore } from "./imageStore";

  let questions: Question[] = [defaultQuestion()];
  let editing: Question | null = null;

  let name: string = "Example Quiz";
  let text: string = "Small description about your quiz";

  let timing: TimingConfig = {
    bonus_score_time: 1000,
    wait_time: 1000,
  };

  async function startQuiz() {
    const config: UploadConfig = {
      basic: { name, text },
      timing,
      questions,
    };

    console.debug("Creating quiz");

    let form = new FormData();
    form.append("config", JSON.stringify(config));
    // TODO: Append images

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

    sendMessage({
      ty: ClientMessageType.Initialize,
      uuid: json.uuid,
    });
  }
</script>

{#if editing}
  <QuestionEditor question={editing} back={() => (editing = null)} />
{:else}
  <button>Back</button>

  <h1>Create Quiz</h1>
  <div>
    <button>Save</button>
    <button>Load</button>
    <button on:click={startQuiz}>Play</button>
  </div>

  <div>
    <input type="text" bind:value={name} />
    <textarea name="" id="" cols="30" rows="10" bind:value={text} />
  </div>

  <QuestionList {questions} bind:editing />
{/if}

<style>
</style>

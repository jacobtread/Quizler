<!-- Component represents a question that is being created -->

<script lang="ts">
  import {
    QuestionDataType,
    type Question,
    type AnswerValue,
  } from "./socket/models";
  import { flip } from "svelte/animate";
  import { imageStore, selectImage } from "./imageStore";

  // The question that is being created
  export let question: Question;
  export let back: () => void;

  /**
   * Handle changes between types to ensure that the
   * question has the relevant fields for that type
   */
  function onTypeChange(event: Event) {
    const target: HTMLSelectElement = event.target as HTMLSelectElement;

    const oldQuestion: Question = question;
    const newValue = target.value as QuestionDataType;

    if (newValue == QuestionDataType.Single) {
      let answers: AnswerValue[] = [];

      if (
        oldQuestion.ty == QuestionDataType.Single ||
        oldQuestion.ty == QuestionDataType.Multiple
      ) {
        answers = oldQuestion.answers;
      }

      question = {
        ...oldQuestion,
        ty: newValue,
        answers,
      };
    } else if (newValue == QuestionDataType.Multiple) {
      let answers: AnswerValue[] = [];
      let min: number = 1;
      let max: number = 1;

      if (
        oldQuestion.ty == QuestionDataType.Single ||
        oldQuestion.ty == QuestionDataType.Multiple
      ) {
        answers = oldQuestion.answers;

        if (oldQuestion.ty == QuestionDataType.Multiple) {
          min = oldQuestion.min;
          max = oldQuestion.max;
        }
      }

      question = {
        ...oldQuestion,
        ty: newValue,
        answers,
        min,
        max,
      };
    }
  }

  function onChangeMin() {
    if (question.ty === QuestionDataType.Multiple) {
      if (question.max < question.min) {
        question.max = question.min;
      }
    }
  }

  function addAnswer() {
    let nextId: number = 0;

    for (const answer of question.answers) {
      if (answer.id >= nextId) {
        nextId = answer.id + 1;
      }
    }

    question.answers.push({
      id: nextId,
      value: "",
      correct: false,
    });
    question.answers = question.answers;
  }

  function swapAnswer(aIndex, bIndex) {
    let a = question.answers[aIndex];
    let b = question.answers[bIndex];

    // Swap the questions
    question.answers[aIndex] = b;
    question.answers[bIndex] = a;
  }

  let image: string | null = null;

  $: {
    let value = $imageStore.find((value) => value.uuid == question.image);
    if (value) {
      image = value.previewUrl;
    } else {
      image = null;
    }
  }

  async function pickImage() {
    let res = await selectImage();
    question.image = res.uuid;
  }

  function removeImage() {
    question.image = null;
    image = null;
  }
</script>

<div class="editor">
  <button on:click={back}>Back</button>

  <div class="question__img-wrapper" on:click={pickImage}>
    {#if image}
      <img class="question__img" src={image} alt="Uploaded Content" />
    {:else}
      <p>Pick Image</p>
    {/if}
  </div>

  {#if image}
    <button on:click={removeImage}>Remove Image</button>
  {/if}

  <textarea
    class="question__text"
    cols="30"
    rows="10"
    bind:value={question.text}
  />

  <select on:change={onTypeChange}>
    <option value={QuestionDataType.Single}>Single Choice</option>
    <option value={QuestionDataType.Multiple}>Multiple Choice</option>
  </select>

  {#if question.ty == QuestionDataType.Single || question.ty == QuestionDataType.Multiple}
    <!-- Min/max choice decision for multiple choice -->
    {#if question.ty == QuestionDataType.Multiple}
      <div>
        <label>
          <span>Min Choices</span>
          <input
            type="number"
            name=""
            id=""
            bind:value={question.min}
            min={1}
            on:change={onChangeMin}
            max={question.answers.length}
          />
        </label>
        <label>
          <span>Max Choices</span>
          <input
            type="number"
            name=""
            id=""
            bind:value={question.max}
            min={question.min}
            max={question.answers.length}
          />
        </label>
      </div>
    {/if}
    <div class="answers">
      {#each question.answers as answer, index (answer.id)}
        <div class="answer" animate:flip={{ duration: 500 }}>
          <div class="answer__move">
            <button
              disabled={index <= 0}
              class="answer__move__dir"
              on:click={() => swapAnswer(index, index - 1)}
            >
              &uarr;
            </button>
            <button
              disabled={index + 1 >= question.answers.length}
              class="answer__move__dir"
              on:click={() => swapAnswer(index, index + 1)}
            >
              &darr;
            </button>
          </div>

          <input
            class="answer__check"
            type="checkbox"
            bind:checked={answer.correct}
          />
          <input
            class="answer__question"
            type="text"
            bind:value={answer.value}
          />
        </div>
      {/each}
      <button on:click={addAnswer}>Add Answer</button>
    </div>
  {/if}
</div>

<style>
  .question {
  }

  .question__text {
    display: block;
    width: 100%;
    margin-bottom: 1rem;
    resize: vertical;
  }

  .answers {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .answer {
    display: flex;
  }

  .answer__move {
    display: grid;
    grid-template-rows: 1fr 1fr;
    gap: 0.5rem;
  }

  .answer__move__dir {
    font-size: 1.5rem;
    transition: all 0.5s ease;
  }

  .answer__question {
    flex: auto;
  }

  .question__img-wrapper {
    max-height: 50vh;
    width: 100%;
    height: 50vh;
    overflow: hidden;
    position: relative;
    margin-bottom: 1rem;
    display: grid;
    justify-content: center;
    align-items: center;
  }

  .question__img {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    height: 100%;
    aspect-ratio: auto;
    z-index: -1;
  }
</style>

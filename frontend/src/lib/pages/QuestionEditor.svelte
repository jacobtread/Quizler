<!-- Component represents a question that is being created -->

<script lang="ts">
  import {
    QuestionType,
    type Question,
    type AnswerValue
  } from "$lib/socket/models";
  import { flip } from "svelte/animate";
  import { imagePreviewStore, selectImage } from "$stores/imageStore";
  import TimeInput from "$lib/components/TimeInput.svelte";
  import {
    MAX_ANSWER_TIME,
    MAX_BONUS_TIME,
    MIN_ANSWER_TIME,
    MIN_BONUS_TIME
  } from "$lib/constants";
  import { randomRange } from "$lib/utils";
  import { saveQuestion } from "$lib/stores/createStore";
  import { confirmDialog } from "$lib/stores/dialogStore";
  import { setCreate } from "$lib/router";
  import ImageStorage from "$lib/components/ImageStorage.svelte";
  import Back from "$lib/assets/icons/back.svg";
  import Delete from "$lib/assets/icons/cross.svg";
  import ArrowUp from "$lib/assets/icons/arrowup.svg";
  import ArrowDown from "$lib/assets/icons/arrowdown.svg";
  import Checkbox from "$lib/components/Checkbox.svelte";

  export let question: Question;

  async function back() {
    const result = await confirmDialog(
      "Confirm Back",
      "Are you sure you want to go back? You will loose any unsaved progress"
    );

    if (!result) return;

    setCreate();
  }

  /**
   * Handle changes between types to ensure that the
   * question has the relevant fields for that type
   */
  function onTypeChange(event: Event) {
    const target: HTMLSelectElement = event.target as HTMLSelectElement;

    const oldQuestion: Question = question;
    const newValue = target.value as QuestionType;

    if (newValue == QuestionType.Single) {
      let answers: AnswerValue[] = [];

      if (
        oldQuestion.ty == QuestionType.Single ||
        oldQuestion.ty == QuestionType.Multiple
      ) {
        answers = oldQuestion.answers;
      }

      question = {
        ...oldQuestion,
        ty: newValue,
        answers
      };
    } else if (newValue == QuestionType.Multiple) {
      let answers: AnswerValue[] = [];
      let min = 1;
      let max = 1;

      if (
        oldQuestion.ty == QuestionType.Single ||
        oldQuestion.ty == QuestionType.Multiple
      ) {
        answers = oldQuestion.answers;

        if (oldQuestion.ty == QuestionType.Multiple) {
          min = oldQuestion.min;
          max = oldQuestion.max;
        }
      }

      question = {
        ...oldQuestion,
        ty: newValue,
        answers,
        min,
        max
      };
    }
  }

  function onChangeMin() {
    if (question.ty === QuestionType.Multiple) {
      if (question.max < question.min) {
        question.max = question.min;
      }
    }
  }

  function addAnswer() {
    let nextId = 0;

    for (const answer of question.answers) {
      if (answer.id >= nextId) {
        nextId = answer.id + 1;
      }
    }

    question.answers.push({
      id: nextId,
      value: "",
      correct: false
    });
    question.answers = question.answers;
  }

  function swapAnswer(aIndex: number, bIndex: number) {
    let a = question.answers[aIndex];
    let b = question.answers[bIndex];

    // Swap the questions
    question.answers[aIndex] = b;
    question.answers[bIndex] = a;
  }

  let image: string | null = null;

  $: {
    // Handle displaying image previews
    if (question.image !== null) {
      let imagePreview = $imagePreviewStore[question.image];
      if (imagePreview) {
        image = imagePreview;
      } else {
        image = null;
      }
    }
  }

  async function pickImage() {
    let res = await selectImage();
    // Handle canceling select image
    if (res === null) return;

    question.image = res.uuid;
  }

  function removeImage(event: Event) {
    event.stopPropagation();
    question.image = null;
    image = null;
  }

  function onImageKeyPress(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === "NumpadEnter") {
      pickImage();
    }
  }

  function removeAnswer(index: number) {
    question.answers = question.answers.filter(
      (_, valueIndex) => valueIndex != index
    );
  }

  function shuffleAnswers() {
    const shuffleCount = randomRange(1, question.answers.length);
    let changes = 0;
    while (changes < shuffleCount) {
      const first = randomRange(0, question.answers.length - 1);
      const second = randomRange(0, question.answers.length - 1);
      if (first !== second) {
        swapAnswer(first, second);
        changes++;
      }
    }
  }

  function save() {
    saveQuestion(question);
    setCreate();
  }
</script>

<main class="main">
  <header class="header btn-row">
    <button on:click={back} class="btn btn--icon">
      <img src={Back} alt="Back" />
      Back
    </button>
    <button on:click={save} class="btn"> Save </button>
  </header>

  <div
    tabindex="0"
    role="button"
    class="question__img-wrapper"
    on:click={pickImage}
    on:keypress={onImageKeyPress}
  >
    {#if image}
      <img class="question__img" src={image} alt="Uploaded Content" />

      <button class="remove" on:click={removeImage}> Click to remove </button>
    {:else}
      <p>Pick Image</p>
    {/if}
  </div>

  <div class="field">
    <p class="field__name">Question</p>
    <p class="field__desc">Enter the question to ask the players</p>
    <textarea
      class="question__text input"
      cols="30"
      rows="10"
      bind:value={question.text}
    />
  </div>

  <div class="field-group">
    <div class="field">
      <p class="field__name">Question Type</p>
      <p class="field__desc">The type of question to present</p>
      <select on:change={onTypeChange} class="input">
        <option value={QuestionType.Single}>Single Choice</option>
        <option value={QuestionType.Multiple}>Multiple Choice</option>
      </select>
    </div>
    <!-- Min/max choice decision for multiple choice -->
    {#if question.ty == QuestionType.Multiple}
      <label class="field">
        <span class="field__name">Min Choices</span>
        <p class="field__desc">
          The minimum number of answers that must be selected to get a pass
        </p>
        <input
          class="input"
          type="number"
          bind:value={question.min}
          min={1}
          on:change={onChangeMin}
          max={question.answers.length}
        />
      </label>
      <label class="field">
        <span class="field__name">Desired Choices</span>
        <p class="field__desc">
          The total number of answers the player must choose to get a full
          score. Can be the same as the minimum choices
        </p>
        <input
          class="input"
          type="number"
          bind:value={question.max}
          min={question.min}
          max={question.answers.length}
        />
      </label>
    {/if}
  </div>

  {#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
    <div class="answers">
      {#each question.answers as answer, index (answer.id)}
        <div class="answer" animate:flip={{ duration: 500 }}>
          <div class="actions">
            <button
              on:click={() => swapAnswer(index, index - 1)}
              disabled={index <= 0}
              class="btn btn--icon-only btn--surface btn-small"
            >
              <img src={ArrowUp} alt="Move Down" />
            </button>

            <button
              on:click={() => swapAnswer(index, index + 1)}
              disabled={index + 1 >= question.answers.length}
              class="btn btn--icon-only btn--surface btn-small"
            >
              <img src={ArrowDown} alt="Move Down" />
            </button>
          </div>

          <Checkbox bind:value={answer.correct} />

          <input
            class="answer__question input"
            type="text"
            bind:value={answer.value}
          />

          <button
            disabled={question.answers.length == 1}
            on:click={() => removeAnswer(index)}
            class="btn btn--icon-only"
          >
            <img src={Delete} alt="Back" />
          </button>
        </div>
      {/each}

      <div class="field-group">
        <button
          class="btn"
          on:click={addAnswer}
          disabled={question.answers.length >= 8}
        >
          Add Answer
        </button>
        <button
          class="btn"
          on:click={shuffleAnswers}
          disabled={question.answers.length <= 1}
        >
          Shuffle
        </button>
      </div>
    </div>
  {/if}

  <div class="group">
    <h2 class="group__title">Timing</h2>
    <p class="group__desc">
      Below you can configuring the timing for different events
    </p>
    <div class="group__value field-group">
      <div class="field">
        <span class="field__name">Answer Time</span>
        <p class="field__desc">Time the players have to answer the question</p>
        <TimeInput
          bind:value={question.answer_time}
          min={MIN_ANSWER_TIME}
          max={MAX_ANSWER_TIME}
        />
      </div>

      <div class="field">
        <span class="field__name">Bonus Score Time</span>
        <p class="field__desc">
          Time the players must answer within for bonus scores
        </p>
        <TimeInput
          bind:value={question.bonus_score_time}
          min={MIN_BONUS_TIME}
          max={MAX_BONUS_TIME}
        />
      </div>
    </div>
  </div>

  <div class="group">
    <h2 class="group__title">Scoring</h2>
    <p class="group__desc">
      Score is awarded to players based on how quickly the player answers the
      question. You can configure the minimum and maximum values for this below
    </p>
    <div class="group__value field-group">
      <label class="field">
        <span class="field__name">Min Score</span>
        <p class="field__desc">
          The minimum amount of score to award for this question
        </p>
        <input
          class="input"
          type="number"
          min={0}
          max={question.scoring.max_score}
          bind:value={question.scoring.min_score}
        />
      </label>
      <label class="field">
        <span class="field__name">Max Score</span>
        <p class="field__desc">
          The maximum amount of score to award for this question
        </p>
        <input
          class="input"
          type="number"
          min={question.scoring.min_score}
          max={1000}
          bind:value={question.scoring.max_score}
        />
      </label>
      <label class="field">
        <span class="field__name">Bonus Score</span>
        <p class="field__desc">
          The amount of score to add for being within the bonus time
        </p>
        <input
          class="input"
          type="number"
          min={0}
          max={1000}
          bind:value={question.scoring.bonus_score}
        />
      </label>
    </div>
  </div>
</main>

<ImageStorage />

<style lang="scss">
  @import "../assets/scheme.scss";

  .header {
    position: sticky;
    top: 0;
    left: 0;
    padding: 1rem 0;
    background-color: $appBackground;
    z-index: 1;
  }

  .field-group {
    display: flex;
    flex-flow: row wrap;
    gap: 1rem;
    width: 100%;
    margin-bottom: 1rem;

    .field {
      margin-bottom: 0;
      flex: auto;
    }
  }

  .group {
    &__title {
      color: #ffffff;
      margin-bottom: 0.5rem;
    }

    &__desc {
      margin-bottom: 1rem;
    }
  }

  .main {
    height: 100%;
    padding: 1rem;
    overflow: auto;
    padding-top: 0;
  }

  .question__text {
    display: block;
    width: 100%;
    resize: vertical;
  }

  .actions {
    display: flex;
    flex-flow: column;
  }

  .answers {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .answer {
    display: flex;
    gap: 1rem;
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

  .remove {
    position: absolute;
    left: 0;
    top: 0;
    opacity: 0;
    width: 100%;
    height: 100%;
    transition: opacity 0.15s ease;
    font-size: 1rem;
    background-color: rgba($color: #000000, $alpha: 0.7);
    border: none;
  }

  .remove:hover {
    opacity: 1;
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
</style>

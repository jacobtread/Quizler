<!-- Component represents a question that is being created -->

<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import { normalizeQuestion, saveQuestion } from "$stores/createStore";
  import { confirmDialog } from "$stores/dialogStore";
  import { setCreate } from "$stores/state";

  import * as constants from "$lib/constants";

  import ImageEditor from "$components/editor/ImageEditor.svelte";
  import Answers from "$components/editor/Answers.svelte";
  import TimeInput from "$components/TimeInput.svelte";
  import Back from "$components/icons/Back.svelte";

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
  function onTypeChange() {
    question = normalizeQuestion(question);
  }

  function maxCorrect(): number {
    let correct = 0;
    for (const answer of question.answers) {
      if (answer.correct) correct += 1;
    }
    return correct;
  }

  function save() {
    // TODO: Precheck question

    saveQuestion(question);
    setCreate();
  }

  $: {
    if (question.ty === QuestionType.Multiple) {
      let max = maxCorrect();
      if (question.max > max) question.max = max;
      if (question.max < question.min) question.max = question.min;
    }
  }
</script>

<main class="main">
  <header class="header btn-row">
    <button on:click={back} class="btn btn--icon">
      <Back />
      Back
    </button>
    <button on:click={save} class="btn"> Save </button>
  </header>

  <ImageEditor {question} />

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
      <select bind:value={question.ty} on:change={onTypeChange} class="input">
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

  <Answers bind:question />

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
          min={constants.MIN_ANSWER_TIME}
          max={constants.MAX_ANSWER_TIME}
        />
      </div>

      <div class="field">
        <span class="field__name">Bonus Score Time</span>
        <p class="field__desc">
          Time the players must answer within for bonus scores
        </p>
        <TimeInput
          bind:value={question.bonus_score_time}
          min={constants.MIN_BONUS_TIME}
          max={constants.MAX_BONUS_TIME}
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

<style lang="scss">
  @import "../../assets/scheme.scss";

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

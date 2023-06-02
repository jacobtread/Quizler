<!-- Component represents a question that is being created -->

<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import { normalizeQuestion } from "$stores/createStore";

  import * as constants from "$lib/constants";

  import ImageEditor from "$components/editor/ImageEditor.svelte";
  import Answers from "$components/editor/Answers.svelte";
  import { slide } from "svelte/transition";
  import Cog from "$lib/components/icons/Cog.svelte";
  import QuestionSettings from "$lib/components/editor/QuestionSettings.svelte";

  export let question: Question;
  let settings: boolean = false;
  /**
   * Handle changes between types to ensure that the
   * question has the relevant fields for that type
   */
  function onTypeChange() {
    question = normalizeQuestion(question);
  }

  /**
   * Returns the number of quesitons marked as
   * correct
   */
  function correct(): number {
    let correct = 0;
    for (const answer of question.answers) {
      if (answer.correct) correct += 1;
    }
    return correct;
  }

  $: {
    if (question.ty === QuestionType.Multiple) {
      let max = correct();
      if (question.max > max) question.max = max;
      if (question.max < question.min) question.max = question.min;
    }
  }
</script>

<ImageEditor bind:question />

<button on:click={() => (settings = true)} class="btn btn--icon">
  <Cog />
  <span>Settings</span>
</button>

<div class="field">
  <p class="field__name">Question</p>
  <p class="field__desc">Enter the question to ask the players</p>
  <textarea
    class="question__text input"
    cols="30"
    rows="2"
    maxlength={constants.MAX_QUESTION_LENGTH}
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
        The total number of answers the player must choose to get a full score.
        Can be the same as the minimum choices
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

{#if settings}
  <QuestionSettings bind:question bind:visible={settings} />
{/if}

<style lang="scss">
  @import "../../assets/scheme.scss";

  .main {
    width: 100%;
    height: 100%;
    overflow: auto;
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

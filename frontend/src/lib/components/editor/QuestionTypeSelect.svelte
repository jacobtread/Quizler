<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import {
    QuestionType,
    type Question,
    questionTypeText,
    MultipleMarking,
    multipleMarkingText
  } from "$api/models";
  import * as constants from "$lib/constants";
  import {
    normalizeMarkingType,
    normalizeQuestion
  } from "$lib/stores/createStore";

  export let question: Question;
  export let visible: boolean;

  /**
   * Handle changes between types to ensure that the
   * question has the relevant fields for that type
   */
  function onTypeChange() {
    question = normalizeQuestion(question);
  }

  function onMarkingTypeChange() {
    question = normalizeMarkingType(question);
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
      // if (question.max > max) question.max = max;
      // if (question.max < question.min) question.max = question.min;
    }
  }
</script>

<div class="floating-wrapper">
  <div class="dialog">
    <button
      on:click={() => (visible = false)}
      class="btn btn--icon btn--surface"
    >
      <Close />
      Close
    </button>

    <div class="field-group">
      <div class="field">
        <p class="field__name">Question Type</p>
        <p class="field__desc">The type of question to present</p>
        <select bind:value={question.ty} on:change={onTypeChange} class="input">
          <option value={QuestionType.Single}>Single Choice</option>
          <option value={QuestionType.Multiple}>Multiple Choice</option>
        </select>
      </div>

      <p>{questionTypeText[question.ty]}</p>

      <!-- Min/max choice decision for multiple choice -->
      {#if question.ty == QuestionType.Multiple && question.marking !== undefined}
        <div class="field">
          <p class="field__name">Marking type</p>
          <p class="field__desc">The type of question to present</p>
          <select
            bind:value={question.marking.ty}
            on:change={onMarkingTypeChange}
            class="input"
          >
            {#each Object.values(MultipleMarking) as ty}
              <option value={ty}>{ty}: {multipleMarkingText[ty]}</option>
            {/each}
          </select>
        </div>

        {#if question.marking.ty === MultipleMarking.Partial}
          <label class="field">
            <span class="field__name">Required for partial</span>
            <p class="field__desc">
              The minimum number of correct answers to be considered a partially
              correct answer (Less than this will be considered Incorrect)
            </p>
            <input
              class="input"
              type="number"
              bind:value={question.marking.partial}
              min={1}
              max={question.answers.length}
            />
          </label>
          <label class="field">
            <span class="field__name">Required for correct</span>
            <p class="field__desc">
              The number of correct answers to be considered a completely
              correct answer (Greater than or equal to this will be considered
              Correct)
            </p>
            <input
              class="input"
              type="number"
              bind:value={question.marking.correct}
              min={question.marking.partial}
              max={question.answers.length}
            />
          </label>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .floating-wrapper {
    z-index: 1;
  }

  .dialog {
    background-color: $surface;

    border-radius: 0.5rem;

    width: 100%;
    max-width: 32rem;

    margin: 1rem;
    padding: 1rem;

    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  .optional {
    color: #777;
    margin-left: 0.5rem;
  }

  .field {
    display: block;
    background-color: $surface;
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

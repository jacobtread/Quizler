<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import {
    QuestionType,
    type Question,
    questionTypeText,
    MultipleMarking,
    multipleMarkingText
  } from "$api/models";
  import {
    normalizeMarkingType,
    normalizeQuestion
  } from "$lib/stores/createStore";
  import RadioButton from "../RadioButton.svelte";
  import { slide } from "svelte/transition";

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

      // Clamping min-max values to their requirements
      if (question.min > max) question.min = max;
      if (question.max > max) question.max = max;
      if (question.min > question.max) question.max = question.min;

      const marking = question.marking;
      if (marking !== undefined && marking.ty === MultipleMarking.Partial) {
        // Clamping partial-correct
        if (marking.partial > max) marking.partial = max;
        if (marking.correct > max) marking.correct = max;
        if (marking.correct < marking.partial)
          marking.correct = marking.partial;
      }
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

    <div class="field">
      <p class="field__name">Question Type</p>
      <p class="field__desc">The type of question to use</p>
      <div class="radio">
        {#each Object.values(QuestionType) as ty}
          <div>
            <RadioButton
              on:change={onTypeChange}
              bind:group={question.ty}
              value={ty}
            >
              <b>{ty} Choice</b>: {questionTypeText[ty]}
            </RadioButton>
          </div>
        {/each}
      </div>
    </div>

    <!-- Multiple choice additional settings -->
    {#if question.ty == QuestionType.Multiple}
      <div class="section" transition:slide>
        <div class="section">
          <label class="field field--small">
            <div>
              <span class="field__name">Min</span>
              <p class="field__desc">
                The number answers the players <strong>must</strong> choose
              </p>
            </div>
            <input
              class="input"
              type="number"
              bind:value={question.min}
              min={1}
              max={question.answers.length}
            />
          </label>
          <label class="field field--small">
            <div>
              <span class="field__name">Max</span>
              <p class="field__desc">
                The maximum number answers the players <strong>can</strong> choose
              </p>
            </div>
            <input
              class="input"
              type="number"
              bind:value={question.max}
              min={question.min}
              max={question.answers.length}
            />
          </label>
        </div>

        {#if question.marking !== undefined}
          <!-- Marking type -->
          <div class="field">
            <p class="field__name">Marking type</p>
            <p class="field__desc">How the question should be marked</p>

            <div class="radio">
              {#each Object.values(MultipleMarking) as ty}
                <div>
                  <RadioButton
                    on:change={onMarkingTypeChange}
                    bind:group={question.marking.ty}
                    value={ty}
                  >
                    <b>{ty}</b>: {multipleMarkingText[ty]}
                  </RadioButton>
                </div>
              {/each}
            </div>
          </div>

          <!-- Extra settings for partial marking -->
          {#if question.marking.ty === MultipleMarking.Partial}
            <div class="section" transition:slide>
              <!-- Partial required -->
              <label class="field field--small">
                <div>
                  <span class="field__name">Required for partial</span>
                  <p class="field__desc">
                    The minimum number of correct answers to be considered a
                    partially correct answer
                  </p>
                  <p class="field__note">
                    Less than this will be considered Incorrect
                  </p>
                </div>

                <input
                  class="input"
                  type="number"
                  bind:value={question.marking.partial}
                  min={1}
                  max={question.answers.length}
                />
              </label>
              <!-- Correct required -->
              <label class="field field--small">
                <div>
                  <span class="field__name">Required for correct</span>
                  <p class="field__desc">
                    The number of correct answers to be considered a completely
                    correct answer
                  </p>
                  <p class="field__note">
                    Greater than or equal to this will be considered Correct
                  </p>
                </div>
                <input
                  class="input"
                  type="number"
                  bind:value={question.marking.correct}
                  min={question.marking.partial}
                  max={question.answers.length}
                />
              </label>
            </div>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .group {
  }

  .radio {
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .section {
    display: flex;
    flex-flow: column;
    gap: 1rem;
    border: 1px solid #444;
    padding: 1rem;
    border-radius: 0.25rem;
  }

  .floating-wrapper {
    z-index: 1;
    position: fixed;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }

  .dialog {
    background-color: $surface;

    border-radius: 0.5rem;

    width: 100%;
    max-width: 46rem;

    margin: 1rem;
    padding: 1rem;

    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-flow: column;
    background-color: $surface;
    border-radius: 0.55rem;
    justify-content: space-between;

    &__name {
      font-weight: bold;
      color: #ffffff;
    }

    &__desc {
      color: #cccccc;
      margin-bottom: 0.25rem;
    }

    &--small {
      flex-flow: row;
      gap: 1rem;
      align-items: center;

      .field__desc {
        margin-bottom: 0;
      }

      .input {
        max-width: 6rem;
      }
    }
  }

  @media screen and (max-width: 38rem) {
    .field--small {
      flex-flow: column;
      align-items: stretch;
      gap: 0;

      .input {
        max-width: none;
      }
    }
  }

  @media screen and (max-width: 48rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
  }

  .input {
    display: block;
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

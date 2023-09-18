<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import * as constants from "$lib/constants";

  import { flip } from "svelte/animate";

  import Checkbox from "$components/Checkbox.svelte";
  import Delete from "$components/icons/Delete.svelte";

  export let question: Question;

  function addAnswer() {
    /// Adding questions is only supported by single/multiple choice questions
    if (
      question.ty != QuestionType.Single &&
      question.ty != QuestionType.Multiple
    ) {
      return;
    }

    const answers = question.answers;

    let nextId = 0;
    for (const answer of answers) {
      if (answer.id >= nextId) {
        nextId = answer.id + 1;
      }
    }

    answers.push({
      id: nextId,
      value: "",
      correct: false
    });
    question.answers = answers;
  }

  function addTyperAnswer() {
    if (question.ty != QuestionType.Typer) {
      return;
    }

    question.answers.push("");
    question.answers = question.answers;
  }

  function removeAnswer(index: number) {
    /// Adding questions is only supported by single/multiple choice questions
    if (
      question.ty != QuestionType.Single &&
      question.ty != QuestionType.Multiple &&
      question.ty != QuestionType.Typer
    ) {
      return;
    }

    question.answers.splice(index, 1);
    question.answers = question.answers;
  }
</script>

{#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
  <div class="answers">
    {#each question.answers as answer, index (answer.id)}
      <div
        class="answer"
        animate:flip={{ duration: 200 }}
        class:answer--selected={answer.correct}
      >
        <div class="answer__check">
          <Checkbox bind:value={answer.correct} />
        </div>

        <input
          class="answer__input"
          type="text"
          bind:value={answer.value}
          maxlength={constants.MAX_ANSWER_LENGTH}
        />

        <button
          disabled={question.answers.length == 1}
          on:click={() => removeAnswer(index)}
          class="btn btn--icon-only delete"
        >
          <Delete />
        </button>
      </div>
    {/each}
    {#if question.answers.length < constants.MAX_ANSWERS}
      <button class="btn add" on:click={addAnswer}> Add Answer </button>
    {/if}
  </div>
{:else if question.ty === QuestionType.TrueFalse}
  <div class="answers">
    <label
      class="answer answer--bool"
      class:answer--selected={question.answer === true}
    >
      <input
        class="hidden"
        type="radio"
        bind:group={question.answer}
        value={true}
      />

      <p class="answer__text">True</p>
    </label>

    <label
      class="answer answer--bool"
      class:answer--selected={question.answer === false}
    >
      <input
        class="hidden"
        type="radio"
        bind:group={question.answer}
        value={false}
      />

      <p class="answer__text">False</p>
    </label>
  </div>
{:else if question.ty === QuestionType.Typer}
  <div class="answers">
    {#each question.answers as answer, index}
      <div class="answer">
        <input
          class="answer__input"
          type="text"
          bind:value={answer}
          maxlength={constants.MAX_ANSWER_LENGTH}
        />

        <button
          disabled={question.answers.length == 1}
          on:click={() => removeAnswer(index)}
          class="btn btn--icon-only delete"
        >
          <Delete />
        </button>
      </div>
    {/each}
    {#if question.answers.length < constants.MAX_ANSWERS}
      <button class="btn add" on:click={addTyperAnswer}> Add Answer </button>
    {/if}
  </div>
{/if}

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .hidden {
    display: none;
  }

  .answers {
    // overflow: hidden;
    padding: 0.25rem;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
  }

  .delete {
    background-color: $btnSurfaceBackground;

    &:disabled {
      background-color: $btnSurfaceDisabled;
    }
  }

  .answer {
    background-color: $surface;
    padding: 0.5rem 1rem;
    gap: 1rem;
    border-radius: 0.5rem;

    display: flex;
    align-items: stretch;
    line-height: 1;

    &--selected {
      outline: 2px solid $primary;
    }

    &__check {
      align-self: center;
    }

    &__input {
      display: block;
      width: 100%;
      padding: 1rem;
      border: none;
      background-color: $surfaceLight;
      border-radius: 0.25rem;
      font-size: 1rem;
    }

    &--bool {
      font-weight: bold;
      font-size: 2rem;
      text-align: center;
      padding: 0.5rem;
    }

    &__text {
      display: block;
      width: 100%;
      padding: 0.5rem;
      border-radius: 0.25rem;
      font-size: 1.25rem;
    }
  }

  .add:nth-child(odd):last-child {
    grid-column-start: 1;
    grid-column-end: 3;
  }

  @media screen and (max-width: 48rem) {
    .answers {
      grid-template-columns: 1fr;
    }

    .add:nth-child(odd):last-child {
      grid-column-start: 1;
      grid-column-end: 2;
    }
  }
</style>

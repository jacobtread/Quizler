<script lang="ts">
  import { QuestionType, type Question } from "$api/models";
  import {
    activeQuestion,
    changeQuestionType,
    replaceQuestion
  } from "$lib/stores/createStore";
  import Checkbox from "../Checkbox.svelte";
  import { confirmDialog } from "$lib/stores/dialogStore";
  import FloatingModal from "../FloatingModal.svelte";

  export let question: Question;
  export let visible: boolean;

  async function setQuestionType(ty: QuestionType) {
    if (ty !== question.ty) {
      const answer = await confirmDialog(
        "Confirm change",
        "Are you sure you want to change the question type? Current questions will be lost."
      );
      if (!answer) return;
    }

    question = changeQuestionType(question, ty);

    replaceQuestion(question);
    activeQuestion.set(question);
  }
</script>

<FloatingModal bind:visible>
  <div class="section">
    <h2 class="section__title">Question Type</h2>
    <p class="section__desc">Please select the type of question below</p>

    <div class="types">
      <button
        class="type"
        class:type--selected={question.ty == QuestionType.Single}
        on:click={() => setQuestionType(QuestionType.Single)}
      >
        <p class="type__name">Single Choice</p>
        <p class="type__desc">Players can only select one answer</p>
        <div class="answers">
          <p class="answer answer--correct" />
          <p class="answer" />
          <p class="answer" />
          <p class="answer" />
        </div>
      </button>

      <button
        class="type"
        class:type--selected={question.ty == QuestionType.Multiple}
        on:click={() => setQuestionType(QuestionType.Multiple)}
      >
        <p class="type__name">Multiple Choice</p>
        <p class="type__desc">Players can select multiple answers</p>
        <div class="answers">
          <p class="answer answer--correct" />
          <p class="answer answer--correct" />
          <p class="answer" />
          <p class="answer answer--correct" />
        </div>
      </button>
      <button
        class="type"
        class:type--selected={question.ty == QuestionType.TrueFalse}
        on:click={() => setQuestionType(QuestionType.TrueFalse)}
      >
        <p class="type__name">True / False</p>
        <p class="type__desc">Simple true or false questions</p>
        <div class="answers">
          <p class="answer answer--correct" />
          <p class="answer" />
        </div>
      </button>
      <button
        class="type"
        class:type--selected={question.ty == QuestionType.Typer}
        on:click={() => setQuestionType(QuestionType.Typer)}
      >
        <p class="type__name">Typer</p>
        <p class="type__desc">Players must type out their answer</p>
        <div class="answers">
          <p class="answer" />
        </div>
      </button>
    </div>
  </div>

  {#if question.ty === QuestionType.Typer}
    <div class="section">
      <h2 class="section__title">Settings</h2>
      <p class="section__desc">Below are settings specific to this type</p>
      <div>
        <div class="row">
          <Checkbox bind:value={question.ignore_case} />
          <p>Ignore case when checking if the answer is correct</p>
        </div>
      </div>
    </div>
  {/if}
</FloatingModal>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .row {
    display: flex;
    flex-flow: row;
    gap: 0.5rem;
    align-items: center;
  }

  .types {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .type {
    text-align: left;
    background-color: $surface;
    border: none;
    padding: 1rem;
    border: 1px solid $surfaceLight;
    border-radius: 0.25rem;
    transition: border-color 0.25s ease;
    cursor: pointer;

    &--selected {
      border-color: $primary;
    }

    &:hover {
      border-color: $surfaceLighter;
    }

    &--selected:hover {
      border-color: $primaryLighter;
    }

    &__name {
      font-size: 1.25rem;
      font-weight: bold;
      color: $textPrimary;
      margin-bottom: 0.25rem;
    }

    &__desc {
      font-size: 1rem;
      margin-bottom: 0.5rem;
    }
  }

  .answers {
    overflow: hidden;
    text-align: center;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.5rem;
  }

  .answer {
    padding: 0.5rem;
    border-radius: 0.25rem;
    background-color: $surfaceLight;
    transition: background-color 0.1s linear;

    &:nth-child(odd):last-child {
      grid-column-start: 1;
      grid-column-end: 3;
    }

    &--correct {
      background-color: $primary;
      color: $textPrimary;
    }
  }

  @media screen and (max-width: 36rem) {
    .types {
      grid-template-columns: 1fr;
    }
  }
</style>

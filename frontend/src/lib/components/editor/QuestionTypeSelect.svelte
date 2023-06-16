<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import { QuestionType, type Question, questionTypeText } from "$api/models";
  import { normalizeQuestion } from "$lib/stores/createStore";
  import RadioButton from "../RadioButton.svelte";
  import { fade, slide } from "svelte/transition";

  export let question: Question;
  export let visible: boolean;

  /**
   * Handle changes between types to ensure that the
   * question has the relevant fields for that type
   */
  function onTypeChange(event: Event) {
    console.log(event);
    question = normalizeQuestion(question);
  }

  $: {
    question = normalizeQuestion(question);
  }
</script>

<div class="floating-wrapper" transition:fade={{ duration: 200 }}>
  <div class="dialog" transition:slide={{ duration: 200 }}>
    <button
      on:click={() => (visible = false)}
      class="btn btn--icon btn--surface"
    >
      <Close />
      Close
    </button>

    <div class="section">
      <h2 class="section__title">Question Type</h2>
      <p class="section__desc">Please select the type of question below</p>

      <div class="types">
        <label
          class="type"
          class:type--selected={question.ty == QuestionType.Single}
        >
          <input
            class="type__input"
            type="radio"
            on:change={onTypeChange}
            bind:group={question.ty}
            value={QuestionType.Single}
            tabindex="0"
          />
          <p class="type__name">Single Choice</p>
          <p class="type__desc">Players can only select one answer</p>
          <div class="answers">
            <p class="answer answer--correct" />
            <p class="answer" />
            <p class="answer" />
            <p class="answer" />
          </div>
        </label>

        <label
          class="type"
          class:type--selected={question.ty == QuestionType.Multiple}
        >
          <input
            class="type__input"
            type="radio"
            on:change={onTypeChange}
            bind:group={question.ty}
            value={QuestionType.Multiple}
          />
          <p class="type__name">Multiple Choice</p>
          <p class="type__desc">Players can select multiple answers</p>
          <div class="answers">
            <p class="answer answer--correct" />
            <p class="answer answer--correct" />
            <p class="answer" />
            <p class="answer answer--correct" />
          </div>
        </label>
      </div>
    </div>
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .types {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .type {
    padding: 1rem;
    border: 2px solid $surfaceLight;
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

    &__input {
      display: none;
    }

    &__name {
      font-size: 1.25rem;
      font-weight: bold;
      color: #fff;
      margin-bottom: 0.25rem;
    }

    &__desc {
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
      color: #fff;
    }
  }

  .radio {
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .section {
    display: flex;
    flex-flow: column;
    border: 1px solid #444;
    padding: 1rem;
    border-radius: 0.25rem;
    &__title {
      color: #ffffff;
      margin-bottom: 0.25rem;
    }

    &__desc {
      margin-bottom: 0.5rem;
    }
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

  @media screen and (max-width: 48rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
  }

  @media screen and (max-width: 36rem) {
    .types {
      grid-template-columns: 1fr;
    }
  }
</style>

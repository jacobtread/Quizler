<!-- Component represents a question that is being created -->

<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import * as constants from "$lib/constants";

  import ImageEditor from "$components/editor/ImageEditor.svelte";
  import Cog from "$lib/components/icons/Cog.svelte";
  import QuestionSettings from "$lib/components/editor/QuestionSettings.svelte";
  import { shuffleArray } from "$lib/utils/utils";
  import { flip } from "svelte/animate";

  import Shuffle from "$components/icons/Shuffle.svelte";

  import Checkbox from "$components/Checkbox.svelte";
  import Delete from "$components/icons/Delete.svelte";
  import QuestionTypeSelect from "$lib/components/editor/QuestionTypeSelect.svelte";
  import Swap from "$lib/components/icons/Swap.svelte";

  export let question: Question;
  let settings: boolean = false;
  let type: boolean = false;

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

  function removeAnswer(index: number) {
    question.answers = question.answers.filter(
      (_, valueIndex) => valueIndex != index
    );
  }

  function shuffleAnswers() {
    question.answers = shuffleArray(question.answers);
  }
</script>

<ImageEditor bind:question />

<div class="actions btn-row">
  <button on:click={() => (settings = true)} class="btn btn--icon">
    <Cog />
    <span>Settings</span>
  </button>

  {#if question.answers !== undefined}
    <button
      class="btn btn--icon"
      on:click={shuffleAnswers}
      disabled={question.answers.length <= 1}
    >
      <Shuffle />
      Shuffle Answers
    </button>
  {/if}

  <button class="btn btn--icon qt" on:click={() => (type = true)}>
    <Swap />
    Change Type

    <span class="qt__type">{question.ty}</span>
  </button>
</div>

<textarea
  class="question__text input"
  cols="30"
  rows="2"
  maxlength={constants.MAX_QUESTION_LENGTH}
  bind:value={question.text}
/>

{#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
  <div class="answers">
    {#each question.answers as answer, index (answer.id)}
      <div class="answer" animate:flip={{ duration: 200 }}>
        <div class="answer__check">
          <Checkbox bind:value={answer.correct} />
        </div>

        <input
          class="answer__question input"
          type="text"
          bind:value={answer.value}
          maxlength={constants.MAX_ANSWER_LENGTH}
        />

        <button
          disabled={question.answers.length == 1}
          on:click={() => removeAnswer(index)}
          class="btn btn--surface btn--icon-only"
        >
          <Delete />
        </button>
      </div>
    {/each}
    {#if question.answers.length < constants.MAX_ANSWERS}
      <button class="btn add" on:click={addAnswer}> Add Answer </button>
    {/if}
  </div>
{/if}

{#if settings}
  <QuestionSettings bind:question bind:visible={settings} />
{/if}

{#if type}
  <QuestionTypeSelect bind:question bind:visible={type} />
{/if}

<style lang="scss">
  @import "../../assets/scheme.scss";

  .qt {
    padding: 0.5rem;

    &__type {
      background-color: $surfaceLight;
      padding: 0.5rem;
      border-radius: 0.5rem;
      margin-left: 0.5rem;
    }
  }
  .actions {
    margin-bottom: 0.5rem;
    display: flex;
    flex-flow: row wrap;
  }

  .answers {
    overflow: hidden;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
  }

  .add:nth-child(odd):last-child {
    grid-column-start: 1;
    grid-column-end: 3;
  }

  .answer {
    background-color: $surface;
    padding: 0.5rem 1rem;
    display: flex;
    align-items: stretch;
    gap: 1rem;
    border-radius: 0.5rem;

    &:nth-child(odd):last-child {
      grid-column-start: 1;
      grid-column-end: 3;
    }

    &__check {
      align-self: center;
      line-height: 0;
    }

    &__question {
      align-self: stretch;
      flex: auto;
    }
  }

  .answer .input {
    display: block;
    width: 100%;
    padding: 1rem;
    border: none;
    background-color: $surfaceLight;
    border-radius: 0.25rem;
    font-size: 1rem;
    line-height: 1;
  }

  @media screen and (max-width: 48rem) {
    .answers {
      grid-template-columns: 1fr;
    }

    .answer:nth-child(odd):last-child,
    .add:nth-child(odd):last-child {
      grid-column-start: 1;
      grid-column-end: 2;
    }

    .actions {
      .btn {
        flex: auto;
      }
    }
  }

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
    margin-bottom: 1rem;
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

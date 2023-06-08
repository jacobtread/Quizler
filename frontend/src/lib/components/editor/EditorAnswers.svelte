<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import * as constants from "$lib/constants";

  import { flip } from "svelte/animate";

  import Checkbox from "$components/Checkbox.svelte";
  import Delete from "$components/icons/Delete.svelte";

  export let question: Question;

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
</script>

{#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
  <div class="answers">
    {#each question.answers as answer, index (answer.id)}
      <div class="answer" animate:flip={{ duration: 200 }}>
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

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .answers {
    overflow: hidden;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
  }

  .answer {
    background-color: $surface;
    padding: 0.5rem 1rem;
    gap: 1rem;
    border-radius: 0.5rem;

    display: flex;
    align-items: stretch;
    line-height: 1;

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

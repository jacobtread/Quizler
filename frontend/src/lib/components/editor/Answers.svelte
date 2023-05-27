<script lang="ts">
  import { flip } from "svelte/animate";

  import { QuestionType, type Question } from "$lib/api/models";

  import { shuffleArray } from "$lib/utils/utils";

  import Checkbox from "$components/Checkbox.svelte";
  import Delete from "$components/icons/Delete.svelte";

  import * as constants from "$lib/constants";

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

  function shuffleAnswers() {
    question.answers = shuffleArray(question.answers);
  }
</script>

{#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
  <div class="answers">
    {#each question.answers as answer, index (answer.id)}
      <div class="answer" animate:flip={{ duration: 500 }}>
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
  </div>

  <div class="field-group">
    <button
      class="btn"
      on:click={addAnswer}
      disabled={question.answers.length >= constants.MAX_ANSWERS}
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
{/if}

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .answers {
    margin-bottom: 1rem;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
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

    .answer:nth-child(odd):last-child {
      grid-column-start: 1;
      grid-column-end: 2;
    }
  }
</style>

<script lang="ts">
  import { flip } from "svelte/animate";

  import { QuestionType, type Question } from "$lib/api/models";

  import { arraySwap, shuffleArray } from "$lib/utils/utils";

  import Checkbox from "$components/Checkbox.svelte";
  import Delete from "$components/icons/Delete.svelte";
  import ArrowUp from "$components/icons/ArrowUp.svelte";
  import ArrowDown from "$components/icons/ArrowDown.svelte";

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

  const moveUp = (index: number) => {
    question.answers = arraySwap(question.answers, index, index - 1);
  };

  const moveDown = (index: number) => {
    question.answers = arraySwap(question.answers, index, index + 1);
  };

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
        <Checkbox bind:value={answer.correct} />

        <div class="answer__actions">
          <button
            on:click={() => moveUp(index)}
            disabled={index <= 0}
            class="btn btn--icon-only btn--surface btn-small"
          >
            <ArrowUp />
          </button>

          <button
            on:click={() => moveDown(index)}
            disabled={index + 1 >= question.answers.length}
            class="btn btn--icon-only btn--surface btn-small"
          >
            <ArrowDown />
          </button>
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
          class="btn btn--icon-only"
        >
          <Delete />
        </button>
      </div>
    {/each}

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
  </div>
{/if}

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .answers {
    display: flex;
    flex-flow: column;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .answer {
    display: flex;
    align-items: stretch;
    gap: 1rem;
  }

  .answer__actions {
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .answer__question {
    align-self: stretch;
    flex: auto;
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

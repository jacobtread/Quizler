<script lang="ts">
  import { QuestionType, type Question } from "$lib/socket/models";

  export let editing: Question | null;
  export let question: Question;
  export let index: number;
  export let length: number;
  export let swapQuestion: (a: number, b: number) => void;
  export let removeQuestion: (a: number) => void;
</script>

<div class="question__head">
  <div class="answer__move">
    <button
      disabled={index <= 0}
      class="answer__move__dir"
      on:click={() => swapQuestion(index, index - 1)}
    >
      &uarr;
    </button>
    <button
      disabled={index + 1 >= length}
      class="answer__move__dir"
      on:click={() => swapQuestion(index, index + 1)}
    >
      &darr;
    </button>
    <button
      disabled={length == 1}
      class="answer__move__dir"
      on:click={() => removeQuestion(index)}
    >
      Del
    </button>
    <button class="answer__move__dir" on:click={() => (editing = question)}>
      Edit
    </button>
  </div>
  <h2>
    {question.text}
  </h2>
</div>

{#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
  <ul class="answers">
    {#each question.answers as answer}
      <li class="answer" data-correct={answer.correct}>
        {answer.value}
      </li>
    {/each}
  </ul>
{/if}

<style>
  .answer[data-correct="true"] {
    background-color: #ff0000;
  }
</style>

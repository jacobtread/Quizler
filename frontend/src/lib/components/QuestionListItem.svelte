<script lang="ts">
  import { QuestionType, type Question } from "$lib/socket/models";

  export let question: Question;
</script>

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

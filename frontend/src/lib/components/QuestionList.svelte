<script lang="ts">
  import QuestionView from "$lib/components/QuestionListItem.svelte";
  import { defaultQuestion } from "$lib/constants";
  import type { Question } from "$lib/socket/models";
  import { randomRange } from "$lib/utils";
  import { flip } from "svelte/animate";
  import Add from "$lib/assets/icons/add.svg";

  export let questions: Question[];
  export let editing: Question | null;

  function addQuestion() {
    let nextId = 0;

    for (const question of questions) {
      if (question.id >= nextId) {
        nextId = question.id + 1;
      }
    }

    const question = defaultQuestion();
    question.id = nextId;

    questions.push(question);
    questions = questions;
  }

  function shuffleQuestions() {
    const shuffleCount = randomRange(1, questions.length);
    let changes = 0;
    while (changes < shuffleCount) {
      const first = randomRange(0, questions.length - 1);
      const second = randomRange(0, questions.length - 1);
      if (first !== second) {
        swapQuestion(first, second);
        changes++;
      }
    }
  }

  function swapQuestion(aIndex: number, bIndex: number) {
    let a = questions[aIndex];
    let b = questions[bIndex];

    // Swap the questions
    questions[aIndex] = b;
    questions[bIndex] = a;
  }

  function removeQuestion(index: number) {
    questions = questions.filter((_, valueIndex) => valueIndex != index);
  }
</script>

<div class="actions">
  <button
    on:click={addQuestion}
    disabled={questions.length >= 50}
    class="icon-button"
  >
    <img src={Add} alt="Back" class="icon-button__img" />
    <span class="icon-button__text"> Add Question</span>
  </button>
  <button
    on:click={shuffleQuestions}
    disabled={questions.length <= 1}
    class="button"
  >
    Shuffle
  </button>
</div>

<ol class="questions">
  {#each questions as question, index (question.id)}
    <li class="question" animate:flip={{ duration: 500 }}>
      <QuestionView
        bind:question
        {index}
        length={questions.length}
        {swapQuestion}
        {removeQuestion}
        bind:editing
      />
    </li>
  {/each}
</ol>

<style>
  .actions {
    display: flex;
    gap: 1rem;
  }
</style>

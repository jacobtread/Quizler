<script lang="ts">
  import QuestionView from "$lib/components/QuestionListItem.svelte";
  import { defaultQuestion } from "$lib/constants";
  import type { Question } from "$lib/socket/models";
  import { flip } from "svelte/animate";

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

<button on:click={addQuestion}>Add Question</button>

<ol class="questions">
  {#each questions as question, index (question.id)}
    <li class="question" animate:flip={{ duration: 500 }}>
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
            disabled={index + 1 >= questions.length}
            class="answer__move__dir"
            on:click={() => swapQuestion(index, index + 1)}
          >
            &darr;
          </button>
          <button
            disabled={questions.length == 1}
            class="answer__move__dir"
            on:click={() => removeQuestion(index)}
          >
            Del
          </button>
          <button
            class="answer__move__dir"
            on:click={() => (editing = question)}
          >
            Edit
          </button>
        </div>
        <h2>
          {question.text}
        </h2>

        <QuestionView bind:question />
      </div>
    </li>
  {/each}
</ol>

<div />

<style>
</style>

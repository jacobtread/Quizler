<script lang="ts">
  import { QuestionType, type Question } from "$lib/socket/models";

  import { removeQuestion, swapQuestion } from "$stores/createStore";
  import { setEditing } from "$stores/state";

  import ArrowUp from "$components/icons/ArrowUp.svelte";
  import ArrowDown from "$components/icons/ArrowDown.svelte";
  import Delete from "$components/icons/Delete.svelte";
  import Edit from "$components/icons/Edit.svelte";

  import { deepCopy } from "$lib/utils/utils";

  export let question: Question;
  export let index: number;
  export let length: number;

  /**
   * Updates the route to the editing route
   * for the current question.
   */
  function edit() {
    // Use a copy of the question for editing
    setEditing(deepCopy(question));
  }
</script>

<div class="question">
  <div class="actions">
    <button
      on:click={() => swapQuestion(index, index - 1)}
      disabled={index <= 0}
      class="btn btn--icon-only btn--surface"
    >
      <ArrowUp />
    </button>

    <button
      on:click={() => swapQuestion(index, index + 1)}
      disabled={index + 1 >= length}
      class="btn btn--icon-only btn--surface"
    >
      <ArrowDown />
    </button>

    <button
      on:click={() => removeQuestion(index)}
      disabled={length == 1}
      class="btn btn--icon-only btn--surface"
    >
      <Delete />
    </button>

    <button on:click={edit} class="btn btn--icon-only btn--surface">
      <Edit />
    </button>
  </div>
  <div class="body">
    <p>
      {question.text}
    </p>

    {#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
      <ul class="answers">
        {#each question.answers as answer}
          <li class="answer" data-correct={answer.correct}>
            {answer.value}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style lang="scss">
  @import "../../assets/scheme.scss";

  .body {
    gap: 1rem;
    display: grid;
    grid-template-columns: 1fr;
    justify-content: space-between;
  }

  .question {
    background-color: $surface;
    padding: 1rem;
    border-radius: 0.5rem;
    display: grid;
    grid-template-columns: min-content auto;
    gap: 1rem;
  }

  .answers {
    list-style-position: inside;
    border-radius: 0.5rem;
    overflow: hidden;
    background-color: $surfaceLight;
  }

  .answer {
    padding: 0.5rem;
  }

  .actions {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.5rem;
  }

  .answer[data-correct="true"] {
    border-radius: 0.5rem;
    border-left: 5px solid $primary;
    color: #fff;
  }
</style>

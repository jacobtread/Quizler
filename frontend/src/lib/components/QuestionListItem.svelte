<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

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

  // Move the question up
  const moveUp = () => swapQuestion(index, index - 1);

  // Move the question down
  const moveDown = () => swapQuestion(index, index + 1);

  // Remove the question
  const remove = () => removeQuestion(index);
</script>

<div class="question">
  <div class="actions">
    <button
      on:click={moveUp}
      disabled={index <= 0}
      class="btn btn--icon-only btn--surface"
    >
      <ArrowUp />
    </button>

    <button
      on:click={moveDown}
      disabled={index + 1 >= length}
      class="btn btn--icon-only btn--surface"
    >
      <ArrowDown />
    </button>

    <button
      on:click={remove}
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
  .question {
    background-color: $surface;
    padding: 1rem;
    border-radius: 0.5rem;
    display: flex;
    gap: 1rem;
  }

  .body {
    flex: auto;
    gap: 1rem;
    display: flex;
    flex-flow: column;
    justify-content: space-between;
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
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .answer[data-correct="true"] {
    border-radius: 0.5rem;
    border-left: 5px solid $primary;
    color: #fff;
  }
</style>

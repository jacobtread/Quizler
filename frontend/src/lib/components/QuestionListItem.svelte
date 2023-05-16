<script lang="ts">
  import { QuestionType, type Question } from "$lib/socket/models";
  import { removeQuestion, swapQuestion } from "$lib/stores/createStore";
  import { setRoute } from "$components/Router.svelte";

  import Delete from "$assets/icons/cross.svg";
  import Edit from "$assets/icons/edit.svg";
  import ArrowUp from "$assets/icons/arrowup.svg";
  import ArrowDown from "$assets/icons/arrowdown.svg";

  import { deepCopy } from "$lib/utils/utils";

  export let question: Question;
  export let index: number;
  export let length: number;

  /**
   * Updates the route to the editing route
   * for the current question.
   */
  function edit() {
    setRoute("Editing", {
      // Use a copy of the question for editing
      question: deepCopy(question)
    });
  }
</script>

<div class="question">
  <div class="actions">
    <button
      on:click={() => swapQuestion(index, index - 1)}
      disabled={index <= 0}
      class="btn btn--icon-only btn--surface"
    >
      <img src={ArrowUp} alt="Move Up" />
    </button>

    <button
      on:click={() => swapQuestion(index, index + 1)}
      disabled={index + 1 >= length}
      class="btn btn--icon-only btn--surface"
    >
      <img src={ArrowDown} alt="Move Down" />
    </button>

    <button
      on:click={() => removeQuestion(index)}
      disabled={length == 1}
      class="btn btn--icon-only btn--surface"
    >
      <img src={Delete} alt="Back" />
    </button>

    <button on:click={edit} class="btn btn--icon-only btn--surface">
      <img src={Edit} alt="Edit" />
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
  @import "../assets/scheme.scss";

  .body {
    display: flex;
    gap: 1rem;
    flex-flow: column;
    justify-content: space-between;
    flex: auto;
  }

  .question {
    background-color: $surface;
    padding: 1rem;
    border-radius: 0.5rem;
    display: flex;
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
    display: flex;
    gap: 0.5rem;
    flex-flow: column;
  }

  .answer[data-correct="true"] {
    border-radius: 0.5rem;
    border-left: 5px solid $primary;
    color: #fff;
  }
</style>

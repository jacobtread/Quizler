<script lang="ts">
  import { QuestionType, type Question } from "$lib/socket/models";
  import { removeQuestion, swapQuestion } from "$lib/stores/createStore";
  import { setEditing } from "$lib/router";
  import Delete from "$lib/assets/icons/cross.svg";
  import Edit from "$lib/assets/icons/edit.svg";
  import ArrowUp from "$lib/assets/icons/arrowup.svg";
  import ArrowDown from "$lib/assets/icons/arrowdown.svg";

  export let question: Question;
  export let index: number;
  export let length: number;
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

    <button
      on:click={() => setEditing(question)}
      class="btn btn--icon-only btn--surface"
    >
      <img src={Edit} alt="Back" />
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

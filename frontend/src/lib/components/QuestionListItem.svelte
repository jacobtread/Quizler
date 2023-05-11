<script lang="ts">
  import { QuestionType, type Question } from "$lib/socket/models";
  import { removeQuestion, swapQuestion } from "$lib/stores/createStore";
  import { setEditing } from "$lib/stores/state";
  import Delete from "$lib/assets/icons/cross.svg";
  import Edit from "$lib/assets/icons/edit.svg";

  export let question: Question;
  export let index: number;
  export let length: number;
</script>

<div class="question">
  <div class="actions">
    <button
      disabled={index <= 0}
      class="action_move button"
      on:click={() => swapQuestion(index, index - 1)}
    >
      &uarr;
    </button>
    <button
      disabled={index + 1 >= length}
      class="action_move button"
      on:click={() => swapQuestion(index, index + 1)}
    >
      &darr;
    </button>

    <button
      on:click={() => removeQuestion(index)}
      disabled={length == 1}
      class="icon-button"
    >
      <img src={Delete} alt="Back" class="icon-button__img" />
    </button>

    <button on:click={() => setEditing(question)} class="icon-button">
      <img src={Edit} alt="Back" class="icon-button__img" />
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

  .button,
  .icon-button {
    background-color: $surfaceLight;
  }

  .icon-button {
    padding-right: 0.5rem;
  }
  .button:disabled,
  .icon-button:disabled {
    background-color: $surfaceLightDisabled;
  }
  .icon-button__img {
    margin: 0 auto;
    width: 24px;
    height: 24px;
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

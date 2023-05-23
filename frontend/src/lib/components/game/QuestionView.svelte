<script lang="ts">
  import * as socket from "$api/socket";
  import {
    QuestionType,
    type Question,
    type TimerState,
    ClientMessage,
    AnswerType
  } from "$api/models";
  import { formatTime } from "$lib/utils/utils";

  export let timer: TimerState;
  export let question: Question;
  export let answered: boolean;

  export let preloadedImage: HTMLImageElement | null;

  let answers: number[] = [];

  async function doAnswer(index: number) {
    answered = true;

    try {
      await socket.send({
        ty: ClientMessage.Answer,
        answer: {
          ty: AnswerType.Single,
          answer: index
        }
      });
    } catch (e) {
      console.error("Error while attempting to answer", e);
    }
  }

  async function doAnswers() {
    answered = true;
    try {
      await socket.send({
        ty: ClientMessage.Answer,
        answer: {
          ty: AnswerType.Multiple,
          answers
        }
      });
    } catch (e) {
      console.error("Error while attempting to answer", e);
    }
  }

  function select(index: number) {
    if (answers.includes(index)) {
      answers = answers.filter((value) => value != index);
    } else {
      answers.push(index);
      answers = answers;
    }
  }

  function preloadChild(target: HTMLElement, elm: HTMLImageElement) {
    target.appendChild(elm);
  }
</script>

<p class="time">{formatTime(timer)}</p>

<main class="main">
  {#if preloadedImage !== null}
    <div class="image-wrapper" use:preloadChild={preloadedImage} />
  {/if}

  <div class="content">
    <p class="text">{question.text}</p>
    {#if question.ty === QuestionType.Single}
      <div class="answers">
        {#each question.answers as answer, index}
          <button class="answer btn" on:click={() => doAnswer(index)}>
            {answer.value}
          </button>
        {/each}
      </div>
    {:else if question.ty === QuestionType.Multiple}
      <div class="answers">
        {#each question.answers as answer, index}
          <button
            class="answer btn"
            class:answer--checked={answers.includes(index)}
            disabled={answers.length >= question.max &&
              !answers.includes(index)}
            on:click={() => select(index)}
          >
            {answer.value}
          </button>
        {/each}
      </div>
      <button
        class="btn submit"
        on:click={doAnswers}
        disabled={answers.length < question.min}
      >
        Submit
      </button>
    {/if}
  </div>
</main>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .main {
    display: flex;
    flex-flow: column;

    gap: 1rem;
    height: 100%;
    overflow: hidden;
    padding: 1rem;
  }

  .time {
    position: fixed;
    right: 1rem;
    top: 1rem;
    color: $primary;
    font-weight: bold;
    font-size: 2rem;
  }

  .text {
    margin-bottom: 1rem;
    color: #ffffff;
    font-size: 1.25rem;
  }

  .image-wrapper {
    flex: auto;
    overflow: hidden;
    position: relative;
  }

  .image-wrapper ~ .content {
    flex: none;

    .text {
      margin-top: 0;
    }
  }

  .content {
    display: flex;
    flex-flow: column;
    min-height: 25vh;

    flex: auto;

    .text {
      margin-top: 4rem;
    }
  }

  .answers {
    flex: auto;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
  }

  .answer {
    padding: 1rem;
    word-wrap: break-word;

    &:nth-child(odd):last-child {
      grid-column-start: 1;
      grid-column-end: 3;
    }

    &--checked {
      background-color: $primary;

      &:hover {
        background-color: $primary;
      }
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  .submit {
    margin-top: 1rem;
    display: block;
  }
</style>

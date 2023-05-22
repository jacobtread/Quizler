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
  import { formatImageUrl } from "$api/http";
  import type { GameData } from "$pages/Game.svelte";

  export let gameData: GameData;
  export let timer: TimerState;
  export let question: Question;
  export let answered: boolean;

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
</script>

<p class="time">{formatTime(timer)}</p>

<main class="main">
  {#if question.image !== null}
    <div class="image-wrapper">
      <img
        class="image"
        src={formatImageUrl(gameData.token, question.image.uuid)}
        data-fit={question.image.fit}
        alt={question.text}
      />
    </div>
  {/if}

  <div class="content" data-image={question.image !== null}>
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

  .image {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    aspect-ratio: auto;
    z-index: -1;

    // Fit for width
    &[data-fit="Width"] {
      width: 100%;
    }

    // Fit for height
    &[data-fit="Height"] {
      height: 100%;
    }

    // Fit for containing whole image
    &[data-fit="Contain"] {
      height: 100%;
      width: 100%;
      object-fit: contain;
    }

    // Fit for covering available space
    &[data-fit="Cover"] {
      height: 100%;
      width: 100%;
      object-fit: cover;
    }
  }

  .content {
    display: flex;
    flex-flow: column;
    min-height: 25vh;

    &[data-image="false"] {
      flex: auto;

      .text {
        margin-top: 4rem;
      }
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

<script lang="ts">
  import * as socket from "$api/socket";
  import {
    QuestionType,
    type Question,
    type TimerState,
    ClientMessage,
    AnswerType,
    HostAction
  } from "$api/models";
  import { formatTime } from "$lib/utils/utils";
  import type { GameData } from "$pages/Game.svelte";
  import { doHostAction } from "$lib/api/actions";

  export let gameData: GameData;
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

  // Sends the skip timer action
  const skip = () => doHostAction(HostAction.Skip);
</script>

<main class="main">
  {#if preloadedImage !== null}
    <div class="image-wrapper" use:preloadChild={preloadedImage} />
  {/if}

  <div class="content">
    <p class="text">{question.text}</p>
    {#if question.ty === QuestionType.Single}
      <div class="answers">
        {#each question.answers as answer, index}
          <button
            data-host={gameData.host}
            class="answer btn btn--surface"
            disabled={gameData.host}
            on:click={() => doAnswer(index)}
          >
            {answer.value}
          </button>
        {/each}
      </div>
    {:else if question.ty === QuestionType.Multiple}
      <div class="answers">
        {#each question.answers as answer, index}
          <button
            data-host={gameData.host}
            class="answer btn btn--surface"
            class:answer--checked={answers.includes(index)}
            disabled={gameData.host ||
              (answers.length >= question.max && !answers.includes(index))}
            on:click={() => select(index)}
          >
            {answer.value}
          </button>
        {/each}
      </div>
      <button
        class="btn btn btn--surface submit"
        on:click={doAnswers}
        disabled={answers.length < question.min}
      >
        Submit
      </button>
    {/if}
  </div>
  <div class="bottom">
    <p class="token">{gameData.token}</p>
    {#if gameData.host}
      <button class="btn btn--surface" on:click={skip}>Skip</button>
    {/if}
    <p class="time">{formatTime(timer)}</p>
  </div>
</main>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .bottom {
    width: 100%;
    background-color: $surface;
    padding: 0.5rem 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 5px solid $surfaceLight;
  }

  .token {
    color: #fff;
    font-weight: bold;
    font-size: 1.5rem;
  }

  .main {
    display: flex;
    flex-flow: column;

    padding-top: 1rem;

    gap: 1rem;
    height: 100%;
    overflow: hidden;
  }

  .time {
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
    padding: 1rem;
    margin-bottom: 0;
    display: flex;
    flex-flow: column;
    background-color: $surface;
    border-radius: 0.5rem;
    margin: 0 1rem;

    flex: auto;
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

    &[data-host="true"]:disabled {
      opacity: 1;
      cursor: not-allowed;
    }
  }

  .submit {
    margin-top: 1rem;
    display: block;
  }
</style>

<script lang="ts">
  import * as socket from "$lib/socket";
  import {
    QuestionType,
    type Question,
    type TimerState,
    ClientMessage,
    AnswerType,
    ServerError
  } from "$lib/socket/models";
  import { formatImageUrl, formatTime } from "$lib/utils/utils";
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
      const error = e as ServerError;
      console.error("Error while attempting to answer", error);
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
      const error = e as ServerError;
      console.error("Error while attempting to answer", error);
    }
  }
</script>

<p class="time">{formatTime(timer)}</p>

<main class="main" data-image={question.image !== null}>
  {#if question.image !== null}
    <div class="image-wrapper">
      <img
        class="image"
        src={formatImageUrl(gameData.token, question.image)}
        alt={question.text}
      />
    </div>
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
          <label class="answer btn">
            <input
              type="checkbox"
              value={index}
              bind:group={answers}
              disabled={answers.length >= question.max &&
                !answers.includes(index)}
            />
            {answer.value}
          </label>
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
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: auto;

    gap: 1rem;
    height: 100%;
    overflow: hidden;
    padding: 1rem;
  }

  .main[data-image="true"] {
    grid-template-rows: auto max-content;
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
    height: 100%;
    aspect-ratio: auto;
    z-index: -1;
  }

  .content {
    display: grid;
    grid-template-rows: min-content auto;
    min-height: 25vh;
  }

  .answers {
    display: grid;
    grid-template-columns: repeat(2, minmax(calc(50% - 0.5rem), 1fr));
    gap: 1rem;
  }

  .answer:nth-child(odd):last-child {
    grid-column-start: 1;
    grid-column-end: 3;
  }

  .answer {
    padding: 1rem;
    word-wrap: break-word;
  }

  .submit {
    margin-top: 1rem;
    display: block;
  }
</style>

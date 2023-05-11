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
  import type { GameData } from "$lib/stores/state";
  import { formatImageUrl, formatTime } from "$lib/utils";

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

<p class="time">Remaining: {formatTime(timer)}</p>

<main class="main">
  <div class="content">
    <p class="name">{question.text}</p>
    <div class="questions">
      {#if question.ty === QuestionType.Single}
        {#each question.answers as answer, index}
          <button on:click={() => doAnswer(index)}>
            {answer.value}
          </button>
        {/each}
      {:else if question.ty === QuestionType.Multiple}
        {#each question.answers as answer, index}
          <label for="">
            <input type="checkbox" bind:value={index} bind:group={answers} />
            {answer.value}
          </label>
        {/each}
        <button on:click={doAnswers}>Submit</button>
      {/if}
    </div>
  </div>
  <div class="image-wrapper">
    {#if question.image !== null}
      <img
        class="image"
        src={formatImageUrl(gameData.token, question.image)}
        alt={question.text}
      />
    {/if}
  </div>
</main>

<style lang="scss">
  @import "../../assets/scheme.scss";

  .main {
    display: flex;
    gap: 1rem;
    flex-flow: column-reverse;
    height: 100%;
    overflow: hidden;
  }

  .time {
    position: fixed;
    left: 0;
    top: 0;
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
</style>

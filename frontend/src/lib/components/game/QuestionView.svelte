<script lang="ts">
  import * as socket from "$lib/socket";
  import {
    QuestionType,
    type Question,
    type TimerState,
    ServerMessage,
    ClientMessage,
    AnswerType
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
    let res = await socket.send(ClientMessage.Answer, {
      answer: {
        ty: AnswerType.Single,
        answer: index
      }
    });

    if (res.ty === ServerMessage.Error) {
      console.error("Error while attempting to answer", res.error);
    }
  }

  async function doAnswers() {
    answered = true;
    let res = await socket.send(ClientMessage.Answer, {
      answer: {
        ty: AnswerType.Multiple,
        answers
      }
    });

    if (res.ty === ServerMessage.Error) {
      console.error("Error while attempting to answer", res.error);
    }
  }
</script>

<div>
  <p>Remaining: {formatTime(timer)}</p>
  {#if question.image !== null}
    <img
      src={formatImageUrl(gameData.token, question.image).toString()}
      alt={question.text}
    />
  {/if}
  <p>{question.text}</p>
  <div>
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

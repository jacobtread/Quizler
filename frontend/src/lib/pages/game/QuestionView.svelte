<script lang="ts">
  import * as socket from "$api/socket";
  import {
    QuestionType,
    type Question,
    ClientMessage,
    AnswerType,
    HostAction,
    type Answer
  } from "$api/models";
  import type { GameData } from "$pages/Game.svelte";
  import { doHostAction } from "$api/actions";
  import { formatTime } from "$lib/utils/utils";
  import QuPreviewImage from "$components/editor/QuPreviewImage.svelte";

  export let gameData: GameData;
  export let timeMs: number;
  export let question: Question;
  export let answered: boolean;

  let answers: number[] = [];
  let typerAnswer: string = "";

  function select(index: number) {
    if (isSelected(index)) {
      answers = answers.filter((value) => value != index);
    } else {
      answers.push(index);
      answers = answers;
    }
  }

  const isSelected = (index: number) => answers.includes(index);

  function canSelect() {
    if (question.ty !== QuestionType.Multiple) return false;
    return answers.length === question.correct_answers;
  }

  // Sends the next state action
  const next = () => doHostAction(HostAction.Next);

  async function answer(answer: Answer) {
    answered = true;
    socket
      .send({
        ty: ClientMessage.Answer,
        answer
      })
      .catch((e) => console.error("Error while attempting to answer", e));
  }

  const answerSingle = (index: number) =>
    answer({
      ty: AnswerType.Single,
      answer: index
    });

  const answerBool = (value: boolean) =>
    answer({
      ty: AnswerType.TrueFalse,
      answer: value
    });

  const answerTyper = () =>
    answer({
      ty: AnswerType.Typer,
      answer: typerAnswer
    });

  const answerMultiple = () =>
    answer({
      ty: AnswerType.Multiple,
      answers
    });
</script>

<main class="main">
  {#if question.image !== null && question.image.preloaded !== undefined}
    <div class="image-wrapper">
      <QuPreviewImage
        fit={question.image.fit}
        preloaded={question.image.preloaded}
      />
    </div>
  {/if}

  <div class="content">
    <p class="text">{question.text}</p>
    {#if question.ty === QuestionType.Single}
      <div class="answers">
        {#each question.answers as answer, index}
          <button
            data-host={gameData.host}
            class="answer btn"
            disabled={gameData.host}
            on:click={() => answerSingle(index)}
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
            class="answer btn"
            class:answer--checked={isSelected(index)}
            disabled={gameData.host || (!isSelected(index) && canSelect())}
            on:click={() => select(index)}
          >
            {answer.value}
          </button>
        {/each}
      </div>
      {#if !gameData.host}
        <button
          class="btn btn submit"
          on:click={answerMultiple}
          disabled={answers.length < 1}
        >
          Submit
        </button>
      {/if}
    {:else if question.ty === QuestionType.TrueFalse}
      <div class="answers">
        <button
          data-host={gameData.host}
          class="answer btn"
          disabled={gameData.host}
          on:click={() => answerBool(true)}
        >
          True
        </button>
        <button
          data-host={gameData.host}
          class="answer btn"
          disabled={gameData.host}
          on:click={() => answerBool(false)}
        >
          False
        </button>
      </div>
    {:else if question.ty === QuestionType.Typer && !gameData.host}
      <input class="input" type="text" bind:value={typerAnswer} />
      <button
        class="btn submit"
        on:click={answerTyper}
        disabled={typerAnswer.length < 1}
      >
        Submit
      </button>
    {/if}
  </div>
  <div class="bottom">
    <p class="token">{gameData.token}</p>
    {#if gameData.host}
      <button class="btn btn--surface" on:click={next}>Skip</button>
    {/if}
    <p class="time">{formatTime(timeMs)}</p>
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
    background-color: $appBackground;
    border: 2px solid $surface;
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

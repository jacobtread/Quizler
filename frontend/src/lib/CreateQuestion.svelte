<!-- Component represents a question that is being created -->

<script lang="ts">
  import {
    QuestionDataType,
    type Question,
    type QuestionIndex,
    type ImageRef,
  } from "./socket/models";

  // The question that is being created
  export let question: Question;
  export let index: number;

  // State determining whether the question is expanded to
  // full visibility
  let expanded = false;

  /**
   * Handle changes between types to ensure that the
   * question has the relevant fields for that type
   */
  function onTypeChange(event: Event) {
    const target: HTMLSelectElement = event.target as HTMLSelectElement;

    const oldQuestion: Question = question;
    const newValue = target.value as QuestionDataType;

    if (newValue == QuestionDataType.Single) {
      let values: string[] = [];
      let answers: QuestionIndex[] = [];

      if (
        oldQuestion.ty == QuestionDataType.Single ||
        oldQuestion.ty == QuestionDataType.Multiple
      ) {
        values = oldQuestion.values;
        answers = oldQuestion.answers;
      }

      question = {
        ty: newValue,
        title: oldQuestion.title,
        text: oldQuestion.text,
        image: oldQuestion.image,
        answer_time: oldQuestion.answer_time,
        scoring: oldQuestion.scoring,
        answers,
        values,
      };
    } else if (newValue == QuestionDataType.Multiple) {
      let values: string[] = [];
      let answers: QuestionIndex[] = [];
      let min: number | null = null;
      let max: number | null = null;

      if (
        oldQuestion.ty == QuestionDataType.Single ||
        oldQuestion.ty == QuestionDataType.Multiple
      ) {
        values = oldQuestion.values;
        answers = oldQuestion.answers;

        if (oldQuestion.ty == QuestionDataType.Multiple) {
          min = oldQuestion.min;
          max = oldQuestion.max;
        }
      }

      question = {
        ty: newValue,
        title: oldQuestion.title,
        text: oldQuestion.text,
        image: oldQuestion.image,
        answer_time: oldQuestion.answer_time,
        scoring: oldQuestion.scoring,
        answers,
        values,
        min,
        max,
      };
    }
  }
</script>

<div class="question" data-expanded={expanded}>
  <!-- Header of the question is a button to expand the content -->
  <button class="question__head" on:click={() => (expanded = !expanded)}>
    <h2 class="question__title">
      <span class="question__title__index">{index + 1}</span>
      {question.title}
    </h2>
  </button>

  {#if expanded}
    <div class="question__body">
      {#if question.image != null}
        <img src={question.image} alt="Uploaded Content" />
      {:else}
        <button>Upload Image</button>
      {/if}
      <textarea
        class="question__text"
        cols="30"
        rows="10"
        bind:value={question.text}
      />

      <select on:change={onTypeChange}>
        <option value={QuestionDataType.Single}>Single Choice</option>
        <option value={QuestionDataType.Multiple}>Multiple Choice</option>
      </select>

      {#if question.ty == QuestionDataType.Single}
        <div class="answers">
          {#each question.values as value}
            <div class="answer">
              <input type="checkbox" name="" id="" />
              {value}
            </div>
          {/each}
        </div>
      {:else if question.ty == QuestionDataType.Multiple}
        <div class="answers">
          {#each question.values as value}
            <div class="answer">
              <input type="checkbox" name="" id="" />
              {value}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .question {
  }

  .question__text {
  }
</style>

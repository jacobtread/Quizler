<!-- Component represents a question that is being created -->

<script lang="ts">
  import { type Question } from "$api/models";

  import * as constants from "$lib/constants";

  import ImageEditor from "$components/editor/ImageEditor.svelte";
  import Cog from "$lib/components/icons/Cog.svelte";
  import { shuffleArray } from "$lib/utils/utils";
  import Delete from "$components/icons/Delete.svelte";

  import QuestionSettings from "$components/editor/QuestionSettings.svelte";
  import QuestionTypeSelect from "$components/editor/QuestionTypeSelect.svelte";
  import EditorAnswers from "$components/editor/EditorAnswers.svelte";
  import Shuffle from "$components/icons/Shuffle.svelte";
  import Swap from "$components/icons/Swap.svelte";
  import { createData, removeQuestion } from "$lib/stores/createStore";

  export let question: Question;
  let settings: boolean = false;
  let type: boolean = false;

  function shuffleAnswers() {
    question.answers = shuffleArray(question.answers);
  }

  function onChange() {
    createData.update((store) => store);
  }

  // Remove the question
  const remove = () => removeQuestion(question);
</script>

<ImageEditor bind:question />

<div class="actions btn-row">
  <button on:click={() => (settings = true)} class="btn btn--icon">
    <Cog />
    <span>Settings</span>
  </button>

  {#if question.answers !== undefined}
    <button
      class="btn btn--icon"
      on:click={shuffleAnswers}
      disabled={question.answers.length <= 1}
    >
      <Shuffle />
      Shuffle Answers
    </button>
  {/if}

  <button class="btn btn--icon qt" on:click={() => (type = true)}>
    <Swap />
    Change Type

    <span class="qt__type">{question.ty}</span>
  </button>

  <button on:click={remove} class="btn btn--icon">
    <Delete />
    <span>Delete</span>
  </button>
</div>

<textarea
  class="text input"
  cols="30"
  rows="2"
  maxlength={constants.MAX_QUESTION_LENGTH}
  bind:value={question.text}
  on:change={onChange}
/>

<EditorAnswers bind:question />

{#if settings}
  <QuestionSettings bind:question bind:visible={settings} />
{/if}

{#if type}
  <QuestionTypeSelect bind:question bind:visible={type} />
{/if}

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .actions {
    margin-bottom: 0.5rem;
    display: flex;
    flex-flow: row wrap;
  }

  .qt {
    padding: 0.5rem;

    &__type {
      background-color: $surfaceLight;
      padding: 0.5rem;
      border-radius: 0.5rem;
      margin-left: 0.5rem;
    }
  }

  .text {
    display: block;
    width: 100%;
    resize: vertical;
    margin-bottom: 1rem;
  }

  .input {
    display: block;
    margin-top: 0.25rem;
    width: 100%;
    padding: 0.5rem;
    border: none;
    background-color: $surfaceLight;
    border-radius: 0.25rem;
    margin-top: 0.5rem;
    font-size: 1rem;
    line-height: 1.5;
  }

  @media screen and (max-width: 64rem) {
    .actions {
      .btn {
        flex: auto;
      }
    }
  }
</style>

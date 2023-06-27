<!-- Component represents a question that is being created -->

<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

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
  import { removeQuestion } from "$lib/stores/createStore";
  import { confirmDialog } from "$lib/stores/dialogStore";

  export let question: Question;
  let settings: boolean = false;
  let type: boolean = false;

  function shuffleAnswers() {
    if (
      question.ty != QuestionType.Single &&
      question.ty != QuestionType.Multiple
    ) {
      return;
    }
    question.answers = shuffleArray(question.answers);
  }

  // Remove the question
  async function remove() {
    const confirmed = await confirmDialog(
      "Confirm Deletion",
      "Are you sure you want to delete this question?"
    );

    if (!confirmed) return;

    removeQuestion(question);
  }
</script>

<ImageEditor bind:question />

<div class="actions btn-row">
  <button on:click={() => (settings = true)} class="btn btn--icon">
    <Cog />
    <span>Settings</span>
  </button>

  <!-- If question type is shufflable include a shuffle button -->
  {#if (question.ty === QuestionType.Single || question.ty === QuestionType.Multiple) && question.answers !== undefined}
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
    Delete
  </button>
</div>

<textarea
  class="text input"
  cols="30"
  rows="2"
  maxlength={constants.MAX_QUESTION_LENGTH}
  bind:value={question.text}
/>

<EditorAnswers bind:question />
<QuestionSettings bind:question bind:visible={settings} />
<QuestionTypeSelect bind:question bind:visible={type} />

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

    height: auto;
    min-height: 2.5rem;
    max-height: 6rem;

    padding: 0.5rem;
    border: none;
    background-color: $surface;
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

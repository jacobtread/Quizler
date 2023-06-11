<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import { QuestionType, type Question, questionTypeText } from "$api/models";
  import { normalizeQuestion } from "$lib/stores/createStore";
  import RadioButton from "../RadioButton.svelte";
  import { fade, slide } from "svelte/transition";

  export let question: Question;
  export let visible: boolean;

  /**
   * Handle changes between types to ensure that the
   * question has the relevant fields for that type
   */
  function onTypeChange(event: Event) {
    console.log(event);
    question = normalizeQuestion(question);
  }

  $: {
    question = normalizeQuestion(question);
  }
</script>

<div class="floating-wrapper" transition:fade={{ duration: 200 }}>
  <div class="dialog" transition:slide={{ duration: 200 }}>
    <button
      on:click={() => (visible = false)}
      class="btn btn--icon btn--surface"
    >
      <Close />
      Close
    </button>

    <div class="field">
      <p class="field__name">Question Type</p>
      <p class="field__desc">The type of question to use</p>
      <div class="radio">
        {#each Object.values(QuestionType) as ty}
          <div>
            <RadioButton
              on:change={onTypeChange}
              bind:group={question.ty}
              value={ty}
            >
              <b>{ty} Choice</b>: {questionTypeText[ty]}
            </RadioButton>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .radio {
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
  }

  .floating-wrapper {
    z-index: 1;
    position: fixed;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }

  .dialog {
    background-color: $surface;

    border-radius: 0.5rem;

    width: 100%;
    max-width: 46rem;

    margin: 1rem;
    padding: 1rem;

    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  @media screen and (max-width: 48rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
  }
</style>

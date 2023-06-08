<script lang="ts">
  import { QuestionType, type Question } from "$api/models";
  import { activeQuestion } from "$lib/stores/createStore";

  import { imagePreviewStore } from "$lib/stores/imageStore";
  import { flip } from "svelte/animate";
  import Image from "../icons/Image.svelte";

  export let question: Question;
  export let index: number;

  let image: string | null = null;

  $: if (question.image !== null) {
    // Handle displaying image previews
    let imagePreview = $imagePreviewStore[question.image.uuid];
    if (imagePreview !== undefined) {
      image = imagePreview;
    } else {
      image = null;
    }
  }

  /**
   * Updates the route to the editing route
   * for the current question.
   */
  function edit() {
    activeQuestion.set(question);
  }

  function onKeydown(event: KeyboardEvent) {
    console.log(event.key);
    if (event.key === "Return" || event.key == "Enter") {
      edit();
    }
  }
</script>

<div
  class="question"
  class:question--active={$activeQuestion !== null &&
    $activeQuestion.id === question.id}
  on:click={edit}
  on:keypress={onKeydown}
>
  <div class="image-wrapper">
    {#if question.image !== null && image !== null}
      <img
        class="image"
        data-fit={question.image.fit}
        src={image}
        alt="Uploaded Content"
      />
    {:else}
      <Image />
    {/if}
  </div>
  <p class="text">
    <span class="question__index">{index + 1} </span>
    {question.text}
  </p>

  {#if question.ty == QuestionType.Single || question.ty == QuestionType.Multiple}
    <div class="answers">
      {#each question.answers as answer (answer.id)}
        <p
          animate:flip={{ duration: 250 }}
          class="answer"
          data-correct={answer.correct}
        />
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .image-wrapper {
    width: 100%;
    height: 2.5rem;
    overflow: hidden;
    position: relative;

    display: flex;
    justify-content: center;
    align-items: center;
  }

  .image {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    aspect-ratio: auto;

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

  .question {
    position: relative;
    background-color: $surface;
    padding: 0.5rem;
    border-radius: 0.5rem;
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
    width: 12rem;

    &:hover {
      outline: 2px solid #666;
    }

    &--active,
    &--active:hover {
      outline: 2px solid $primary;
    }

    &__index {
      display: inline;
      background-color: $surface;

      color: #fff;

      font-weight: bold;
      margin-right: 0.25rem;
    }
  }

  .text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 0.8rem;
  }

  .answers {
    overflow: hidden;
    text-align: center;

    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.5rem;
  }

  .answer {
    padding: 0.5rem;
    border-radius: 0.25rem;
    background-color: $surfaceLight;
    transition: background-color 0.1s linear;

    &:nth-child(odd):last-child {
      grid-column-start: 1;
      grid-column-end: 3;
    }

    &[data-correct="true"] {
      background-color: $primary;
      color: #fff;
    }
  }

  @media screen and (max-width: 64rem) {
    .image-wrapper {
      height: 2rem;
    }

    .question {
      max-width: 12rem;
    }

    .answers {
      display: none;
    }
  }
  @media screen and (max-width: 48rem), (max-height: 48rem) {
    .image-wrapper {
      display: none;
    }

    .question {
      max-width: 6rem;
    }
  }
</style>

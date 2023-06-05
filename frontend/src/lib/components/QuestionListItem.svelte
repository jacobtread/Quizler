<script lang="ts">
  import { QuestionType, type Question } from "$api/models";

  import {
    activeIndex,
    removeQuestion,
    swapQuestion
  } from "$stores/createStore";

  import ArrowUp from "$components/icons/ArrowUp.svelte";
  import ArrowDown from "$components/icons/ArrowDown.svelte";
  import Delete from "$components/icons/Delete.svelte";
  import Edit from "$components/icons/Edit.svelte";

  import { imagePreviewStore } from "$lib/stores/imageStore";
  import { flip } from "svelte/animate";

  export let question: Question;
  export let index: number;
  export let length: number;

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
    activeIndex.set(index);
  }

  // Move the question up
  const moveUp = () => swapQuestion(index, index - 1);

  // Move the question down
  const moveDown = () => swapQuestion(index, index + 1);

  // Remove the question
  const remove = () => removeQuestion(index);
</script>

<div class="question" class:question--active={$activeIndex === index}>
  <div class="actions btn-row btn-row--fill">
    <button
      on:click={moveUp}
      disabled={index <= 0}
      class="btn btn--icon-only btn--surface btn--small"
    >
      <ArrowUp />
    </button>
    <button
      on:click={moveDown}
      disabled={index + 1 >= length}
      class="btn btn--icon-only btn--surface btn--small"
    >
      <ArrowDown />
    </button>
    <button
      on:click={remove}
      disabled={length == 1}
      class="btn btn--icon-only btn--surface btn--small"
    >
      <Delete />
    </button>

    <button
      on:click={edit}
      class="btn btn--icon-only btn--surface btn--small"
      disabled={$activeIndex === index}
    >
      <Edit />
    </button>
  </div>

  {#if question.image !== null && image !== null}
    <div class="image-wrapper">
      <img
        class="image"
        data-fit={"Cover"}
        src={image}
        alt="Uploaded Content"
      />
    </div>
  {/if}
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
  @import "../../assets/scheme.scss";

  .image-wrapper {
    max-height: 50vh;
    width: 100%;
    height: 5rem;
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

    height: 100%;
    width: 100%;
    object-fit: cover;
  }

  .question {
    position: relative;
    background-color: $surface;
    padding: 0.5rem;
    border-radius: 0.5rem;
    display: flex;
    flex-flow: column;
    gap: 0.5rem;
    max-width: 12rem;
    // border: 3px solid $surface;

    &--active {
      outline: 2px solid $primary;
      outline-offset: 0.25rem;
      // border: 3px solid $primary;
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

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  @media screen and (max-width: 64rem) {
    .image-wrapper {
      display: none;
    }

    .question {
      height: 100%;
      overflow: hidden;
      padding: 0.5rem;
      gap: 0.5rem;
    }

    .answers {
      gap: 0.5rem;
    }

    .answer {
      // padding: 0.25rem;
    }
  }
</style>

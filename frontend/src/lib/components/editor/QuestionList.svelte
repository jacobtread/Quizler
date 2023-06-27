<script lang="ts">
  import { flip } from "svelte/animate";

  import { dndzone, type DndEvent } from "svelte-dnd-action";

  import { type Question } from "$api/models";
  import * as constants from "$lib/constants";

  import QuestionListItem from "$lib/components/editor/QuestionListItem.svelte";
  import Add from "$components/icons/Add.svelte";
  import Shuffle from "$components/icons/Shuffle.svelte";

  import {
    createData,
    shuffleQuestions,
    addQuestion,
    activeQuestion
  } from "$stores/createStore";

  function handleDndConsider(e: CustomEvent<DndEvent<Question>>) {
    $createData.questions = e.detail.items;
  }

  function handleDndFinalize(e: CustomEvent<DndEvent<Question>>) {
    $createData.questions = e.detail.items;
  }

  function onClickQuestion(question: Question) {
    activeQuestion.set(question);
  }

  function onKeyQuestion(event: KeyboardEvent, question: Question) {
    if (event.key === "Return" || event.key == "Enter") {
      activeQuestion.set(question);
    }
  }
</script>

<div class="list">
  <button
    on:click={shuffleQuestions}
    disabled={$createData.questions.length <= 1}
    class="btn btn--icon"
  >
    <Shuffle />
    Shuffle
  </button>
  <section
    class="questions"
    use:dndzone={{
      items: $createData.questions,
      flipDurationMs: 200,
      dropTargetStyle: {}
    }}
    on:consider={handleDndConsider}
    on:finalize={handleDndFinalize}
  >
    {#each $createData.questions as question, index (question.id)}
      <div
        class="qw"
        animate:flip={{ duration: 200 }}
        class:qw--active={$activeQuestion !== null &&
          $activeQuestion.id === question.id}
        on:click={() => onClickQuestion(question)}
        on:keydown={(event) => onKeyQuestion(event, question)}
      >
        <QuestionListItem {question} {index} />
      </div>
    {/each}
  </section>
  <button
    on:click={addQuestion}
    disabled={$createData.questions.length >= constants.MAX_QUESTIONS}
    class="btn add btn--icon-only"
  >
    <Add />
  </button>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";
  .qw {
    border-radius: 0.5rem;
    position: relative;

    &:hover {
      outline: 2px solid #666;
    }

    &--active,
    &--active:hover {
      outline: 2px solid $primary;
    }

    &:focus {
      outline: 2px solid #fff;
    }
  }

  .list {
    display: flex;
    flex-flow: column;
    gap: 1rem;

    min-width: 14rem;
  }

  .questions {
    position: relative;
    padding: 1rem;
    overflow: auto;
    flex: auto;
    border: 0.1rem solid $surface;
    border-radius: 0.25rem;

    display: flex;
    gap: 1rem;
    flex-flow: column;
    list-style: none;
  }

  @media screen and (max-width: 64rem) {
    .list {
      flex-flow: row;
      width: auto;
    }

    .questions {
      flex-flow: row;
    }
  }

  @media screen and (max-width: 48rem), (max-height: 48rem) {
    .questions {
      padding: 0;
      align-items: center;
    }
  }
</style>

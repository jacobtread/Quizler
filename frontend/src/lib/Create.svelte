<script lang="ts">
  import { type Question } from "./socket/models";
  import QuestionEditor from "./QuestionEditor.svelte";
  import { defaultQuestion } from "./constants";
  import QuestionList from "./QuestionList.svelte";

  let questions: Question[] = [defaultQuestion()];
  let editing: Question | null = null;

  let title: string = "Example Quiz";
  let text: string = "Small description about your quiz";
</script>

{#if editing}
  <QuestionEditor question={editing} back={() => (editing = null)} />
{:else}
  <button>Back</button>

  <h1>Create Quiz</h1>
  <div>
    <input type="text" bind:value={title} />
    <textarea name="" id="" cols="30" rows="10" bind:value={text} />
  </div>

  <QuestionList {questions} bind:editing />
{/if}

<style>
</style>

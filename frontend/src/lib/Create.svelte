<script lang="ts">
  import CreateQuestion from "./CreateQuestion.svelte";
  import { QuestionDataType, type Question } from "./socket/models";

  const DEFAULT_ANSWER_TIME_MS: number = 1000 * 10; /* 10s */
  const DEFAULT_MIN_SCORE: number = 10;
  const DEFAULT_MAX_SCORE: number = 100;
  const DEFAULT_BONUS_SCORE: number = 150;
  const DEFAULT_QUESTION: Question = {
    ty: QuestionDataType.Single,
    title: "Example Question",
    text: "This is an example question, you should replace this with your first question",
    image: null,
    answer_time: DEFAULT_ANSWER_TIME_MS,
    scoring: {
      min_score: DEFAULT_MIN_SCORE,
      max_score: DEFAULT_MAX_SCORE,
      bonus_score: DEFAULT_BONUS_SCORE,
    },
    answers: [
      { id: 0, value: "Example A", correct: true },
      { id: 1, value: "Example B", correct: false },
      { id: 2, value: "Example C", correct: false },
      { id: 3, value: "Example D", correct: false },
    ],
  };

  const questions: Question[] = [DEFAULT_QUESTION];
</script>

<h1>Create Quiz</h1>

<button>Save</button>
<button>Load</button>

<div class="questions">
  {#each questions as question, index}
    <CreateQuestion {question} {index} />
  {/each}
</div>

<style>
</style>

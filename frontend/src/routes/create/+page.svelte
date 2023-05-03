<script lang="ts">
  import { QuestionDataType, type Question } from "$lib/socket/models";

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
    values: ["Example A", "Example B", "Example C", "Example D"],
    answers: [0],
  };

  const questions: Question[] = [DEFAULT_QUESTION];
</script>

<h1>Create Quiz</h1>

<button>Save</button>
<button>Load</button>

<div class="questions">
  {#each questions as question}
    <div class="question" />
  {/each}
</div>

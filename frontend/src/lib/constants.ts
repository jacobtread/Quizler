import { QuestionType, type Question } from "$lib/socket/models";

export const DEBUG: boolean = import.meta.env.DEV;

const DEFAULT_ANSWER_TIME_MS: number = 1000 * 15; /* 15s */
const DEFAULT_BONUS_TIME_MS: number = 1000 * 3; /* 3s */
const DEFAULT_MIN_SCORE: number = 10;
const DEFAULT_MAX_SCORE: number = 100;
const DEFAULT_BONUS_SCORE: number = 150;

export const MIN_ANSWER_TIME: number = 1000; /* 1s */
export const MAX_ANSWER_TIME: number = 1000 * 60 * 30; /* 30mins */

export const MIN_WAIT_TIME = 1000; /* 1s */
export const MAX_WAIT_TIME = 1000 * 60 * 30; /* 30mins */

export const MIN_BONUS_TIME = 1000; /* 1s */
export const MAX_BONUS_TIME = 1000 * 60 * 30; /* 30mins */

/**
 * Creates a new default question object to use
 *
 * @returns The question
 */
export function defaultQuestion(): Question {
  return {
    id: 0,
    ty: QuestionType.Single,
    text: "This is an example question, you should replace this with your first question",
    image: null,
    answer_time: DEFAULT_ANSWER_TIME_MS,
    bonus_score_time: DEFAULT_BONUS_TIME_MS,
    scoring: {
      min_score: DEFAULT_MIN_SCORE,
      max_score: DEFAULT_MAX_SCORE,
      bonus_score: DEFAULT_BONUS_SCORE
    },
    answers: [
      { id: 0, value: "Example A", correct: true },
      { id: 1, value: "Example B", correct: false },
      { id: 2, value: "Example C", correct: false },
      { id: 3, value: "Example D", correct: false }
    ]
  };
}

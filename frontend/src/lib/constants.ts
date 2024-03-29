import { QuestionType, type Question, NameFiltering } from "$api/models";
import type { AnswerValue, CreateDataRuntime } from "$lib/api/models";
import { v4 } from "uuid";

export const TOKEN_LENGTH: number = 5;

export const MIN_PLAYER_NAME_LENGTH: number = 1;
export const MAX_PLAYER_NAME_LENGTH: number = 30;

export const MAX_TITLE_LENGTH: number = 70;
export const MAX_DESCRIPTION_LENGTH: number = 150;

const DEFAULT_NAME = "Example Quiz";
const DEFAULT_DESCRIPTION = "Small description about your quiz";

const DEFAULT_MIN_SCORE: number = 10;
const DEFAULT_MAX_SCORE: number = 100;

export const MAX_ANSWER_LENGTH: number = 150;

export const MIN_ANSWERS: number = 1;
export const MAX_ANSWERS: number = 8;

export const MAX_QUESTIONS: number = 50;

export const MAX_QUESTION_LENGTH = 400;

const DEFAULT_ANSWER_TIME: number = 1000 * 15; /* 15s */
export const MIN_ANSWER_TIME: number = 1000; /* 1s */
export const MAX_ANSWER_TIME: number = 1000 * 60 * 30; /* 30mins */

const DEFAULT_BONUS_SCORE: number = 150;
const DEFAULT_BONUS_TIME: number = 1000 * 3; /* 3s */

export const MIN_SCORE: number = 0;
export const MAX_MAX_SCORE: number = 10_000;
export const MAX_BONUS_SCORE: number = 10_000;

export const MIN_BONUS_TIME = 1000; /* 1s */
export const MAX_BONUS_TIME = 1000 * 60 * 30; /* 30mins */

export const MIN_MAX_PLAYERS: number = 1; /* 1 max player min */
export const MAX_MAX_PLAYERS: number = 50; /* 50 max player limit to games */

export const MAX_IMAGE_BYTES: number = 1024 * 1024 * 15; /* 15mb */

/**
 * Creates a new create data object with its defaults
 *
 * @returns The create data object
 */
export function defaultCreateData(): CreateDataRuntime {
  return {
    name: DEFAULT_NAME,
    text: DEFAULT_DESCRIPTION,
    max_players: MAX_MAX_PLAYERS,
    filtering: NameFiltering.High,
    questions: [defaultQuestion()]
  };
}

/**
 * Creates a new default question object to use
 *
 * @returns The default question
 */
export function defaultQuestion(): Question {
  return {
    id: v4(),
    ty: QuestionType.Single,
    text: "This is an example question, you should replace this with your first question",
    image: null,
    answer_time: DEFAULT_ANSWER_TIME,
    bonus_score_time: DEFAULT_BONUS_TIME,
    scoring: {
      min_score: DEFAULT_MIN_SCORE,
      max_score: DEFAULT_MAX_SCORE,
      bonus_score: DEFAULT_BONUS_SCORE
    },
    answers: defaultAnswers()
  };
}

/**
 * Creates a new array of default answers for a
 * question to use
 *
 * @returns The array of answers
 */
export function defaultAnswers(): AnswerValue[] {
  return [
    { id: 0, value: "Example A", correct: true },
    { id: 1, value: "Example B", correct: false },
    { id: 2, value: "Example C", correct: false },
    { id: 3, value: "Example D", correct: false }
  ];
}

import { QuestionType, NameFiltering } from "$api/models";
import type { AnswerValue, Question, CreateDataRuntime } from "$api/models";
import { v4 } from "uuid";
import { MAX_MAX_PLAYERS } from "$lib/constants";

const DEFAULT_NAME = "Example Quiz";
const DEFAULT_DESCRIPTION = "Small description about your quiz";

const DEFAULT_MIN_SCORE: number = 10;
const DEFAULT_MAX_SCORE: number = 100;

const DEFAULT_BONUS_SCORE: number = 150;
const DEFAULT_BONUS_TIME: number = 1000 * 3; /* 3s */

const DEFAULT_ANSWER_TIME: number = 1000 * 15; /* 15s */

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

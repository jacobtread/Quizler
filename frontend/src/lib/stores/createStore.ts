import { defaultCreateData, defaultQuestion } from "$lib/constants";
import {
  NameFiltering,
  QuestionType,
  type Question,
  type TimingConfig
} from "$api/models";
import { arraySwap, shuffleArray } from "$lib/utils/utils";
import { writable, type Writable } from "svelte/store";

export interface CreateData {
  name: string;
  text: string;
  max_players: number;
  filtering: NameFiltering;
  timing: TimingConfig;
  questions: Question[];
}

// Store for the current creation data
export const createData: Writable<CreateData> = writable(defaultCreateData());

// The ID for the next question
let nextQuestionId: number = 1;

/**
 * Creates a new default question and inserts it into
 * the questions list
 */
export function addQuestion() {
  createData.update((store) => {
    const question: Question = defaultQuestion();
    question.id = nextQuestionId;
    nextQuestionId++;

    store.questions.push(question);
    return store;
  });
}

/**
 * Swaps the questions at the two provided indexes
 *
 * @param aIndex The first index
 * @param bIndex The second index
 */
export function swapQuestion(aIndex: number, bIndex: number) {
  createData.update((store) => {
    arraySwap(store.questions, aIndex, bIndex);
    return store;
  });
}

/**
 * Removes the question at the provided index
 *
 * @param index The index to remove
 */
export function removeQuestion(index: number) {
  createData.update((store) => {
    store.questions.splice(index, 1);
    return store;
  });
}

/**
 * Randomly shuffles the questions
 */
export function shuffleQuestions() {
  createData.update((store) => {
    shuffleArray(store.questions);
    return store;
  });
}

/**
 * Saves a currenlty editing question by finding
 * it in the questions array and replacing it or
 * pushing it if its missing
 *
 * @param question The question to save
 */
export function saveQuestion(question: Question) {
  createData.update((store) => {
    const index = store.questions.findIndex(
      (value) => value.id === question.id
    );

    if (index === -1) {
      // Add the new question
      store.questions.push(question);
    } else {
      // Replace the existing question
      store.questions[index] = question;
    }

    return store;
  });
}

/**
 * Normalizises the question for its current type. For
 * the multiple choice question it adds the missing
 * min and max fields
 *
 * @param question The question to normalize
 * @returns The question provided
 */
export function normalizeQuestion(question: Question): Question {
  // Create answers if they are missing
  question.answers = question.answers ?? [];

  // Add min max fields if they are missing
  if (question.ty === QuestionType.Multiple) {
    question.min = question.min ?? 1;
    question.max = question.max ?? question.answers.length;
  }

  return question;
}

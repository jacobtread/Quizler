import { defaultCreateData, defaultQuestion } from "$lib/constants";
import {
  NameFiltering,
  QuestionType,
  type Question,
  MultipleMarking
} from "$api/models";
import { arraySwap, randomRange, shuffleArray } from "$lib/utils/utils";
import { writable, type Writable } from "svelte/store";

export interface CreateData {
  name: string;
  text: string;
  max_players: number;
  filtering: NameFiltering;
  questions: Question[];
}

// Store for the current creation data
export const createData: Writable<CreateData> = writable(defaultCreateData());

export const activeIndex: Writable<number> = writable(0);

// The ID for the next question
export let nextQuestionId: number = 1;

export function setCreateData(data: CreateData) {
  createData.set(data);

  nextQuestionId = 0;

  // TODO: Intial data if no question

  activeIndex.update((value) => {
    if (value >= data.questions.length) {
      return 0;
    } else {
      return value;
    }
  });

  for (let i = 0; i < data.questions.length; i++) {
    const question = data.questions[i];
    question.id = nextQuestionId;
    nextQuestionId++;
  }
}

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

    activeIndex.update((value) => {
      if (aIndex === value) {
        return bIndex;
      } else if (bIndex === value) {
        return aIndex;
      } else {
        return value;
      }
    });

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
    activeIndex.update((value) => {
      if (index === value) {
        return 0;
      } else {
        return value;
      }
    });

    store.questions.splice(index, 1);
    return store;
  });
}

/**
 * Randomly shuffles the questions
 */
export function shuffleQuestions() {
  createData.update((store) => {
    const shuffleCount = randomRange(1, store.questions.length / 2);
    let changes = 0;
    while (changes < shuffleCount) {
      const first = randomRange(0, store.questions.length - 1);
      const second = randomRange(0, store.questions.length - 1);
      if (first !== second) {
        swapQuestion(first, second);
        changes++;
      }
    }
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
    question.marking = question.marking ?? { ty: MultipleMarking.Exact };
    normalizeMarkingType(question);
  }

  return question;
}

export function normalizeMarkingType(question: Question): Question {
  if (question.ty === QuestionType.Multiple) {
    const marking = question.marking;

    if (marking.ty === MultipleMarking.Partial) {
      marking.partial = marking.partial ?? 1;
      marking.correct = marking.correct ?? question.answers.length;
    }
  }
  return question;
}

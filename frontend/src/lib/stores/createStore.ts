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

export const createData: Writable<CreateData> = writable(defaultCreateData());

let nextQuestionId: number = 1;

export function addQuestion() {
  createData.update((store) => {
    const question: Question = defaultQuestion();
    question.id = nextQuestionId;
    nextQuestionId++;

    store.questions.push(question);
    return store;
  });
}

export function swapQuestion(aIndex: number, bIndex: number) {
  createData.update((store) => {
    arraySwap(store.questions, aIndex, bIndex);
    return store;
  });
}

export function removeQuestion(index: number) {
  createData.update((store) => {
    store.questions.splice(index, 1);
    return store;
  });
}

export function shuffleQuestions() {
  createData.update((store) => {
    shuffleArray(store.questions);
    return store;
  });
}

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

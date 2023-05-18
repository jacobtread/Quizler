import { defaultCreateData, defaultQuestion } from "$lib/constants";
import { NameFiltering, type Question, type TimingConfig } from "$api/models";
import { randomRange } from "$lib/utils/utils";
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
    const questions: Question[] = store.questions;

    // Get the questions
    const a: Question = questions[aIndex];
    const b: Question = questions[bIndex];

    // Handle the indexes not existing
    if (a !== undefined || b !== undefined) {
      // Swap the questions
      questions[aIndex] = b;
      questions[bIndex] = a;
    }

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
    const questions: Question[] = store.questions;
    const shuffleCount: number = randomRange(1, questions.length);
    let changes = 0;
    while (changes < shuffleCount) {
      const first = randomRange(0, questions.length - 1);
      const second = randomRange(0, questions.length - 1);
      if (first !== second) {
        swapQuestion(first, second);
        changes++;
      }
    }
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

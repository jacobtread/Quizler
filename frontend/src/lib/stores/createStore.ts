import { MAX_MAX_PLAYERS, defaultQuestion } from "$lib/constants";
import type { Question, TimingConfig } from "$lib/socket/models";
import { randomRange } from "$lib/utils";
import { writable, type Writable } from "svelte/store";

export interface CreateData {
  name: string;
  text: string;
  max_players: number;
  timing: TimingConfig;
  questions: Question[];
}

export const createData: Writable<CreateData> = writable({
  name: "Example Quiz",
  text: "Small description about your quiz",
  max_players: MAX_MAX_PLAYERS,
  timing: {
    wait_time: 1000 * 10
  },
  questions: [defaultQuestion()]
});

let nextQuestionId: number = 1;

export function addQuestion() {
  createData.update((store) => {
    const question = defaultQuestion();
    question.id = nextQuestionId;
    nextQuestionId++;

    store.questions.push(question);
    return store;
  });
}

export function swapQuestion(aIndex: number, bIndex: number) {
  createData.update((store) => {
    const questions = store.questions;

    // Get the questions
    const a = questions[aIndex];
    const b = questions[bIndex];

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
    const questions = store.questions;
    const shuffleCount = randomRange(1, questions.length);
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

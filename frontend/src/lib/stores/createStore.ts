import {
  defaultAnswers,
  defaultCreateData,
  defaultQuestion
} from "$lib/constants";
import {
  QuestionType,
  type Question,
  type CreateDataRuntime
} from "$api/models";
import { arraySwap, randomRange } from "$lib/utils/utils";
import { writable, type Writable } from "svelte/store";

// Store for the current creation data
export const createData: Writable<CreateDataRuntime> = writable(
  defaultCreateData()
);
export const activeQuestion: Writable<Question | null> = writable(null);

activeQuestion.subscribe(() => {
  forceUpdateCreate();
});

export function forceUpdateCreate() {
  createData.update((store) => store);
}

export function setCreateData(data: CreateDataRuntime) {
  createData.set(data);

  if (data.questions.length > 0) {
    activeQuestion.set(data.questions[0]);
  } else {
    activeQuestion.set(null);
  }
}

/**
 * Creates a new default question and inserts it into
 * the questions list
 */
export function addQuestion() {
  createData.update((store) => {
    const question: Question = defaultQuestion();
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

export function removeQuestion(question: Question) {
  createData.update((store) => {
    const index = store.questions.findIndex(
      (value) => value.id === question.id
    );

    activeQuestion.update((value) => {
      if (value === null || value.id === question.id) {
        return null;
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
export function replaceQuestion(question: Question) {
  createData.update((store) => {
    const index = store.questions.findIndex(
      (value) => value.id === question.id
    );

    if (index !== -1) {
      // Replace the existing question
      store.questions[index] = question;
    }

    return store;
  });
}

export function changeQuestionType(
  question: Question,
  target: QuestionType
): Question {
  // If the question is of the same type no change should be made
  if (target === question.ty) {
    return question;
  }

  // Create a copy of the basic question details
  const base: Question = {
    id: question.id,
    text: question.text,
    image: question.image,
    answer_time: question.answer_time,
    bonus_score_time: question.bonus_score_time,
    scoring: question.scoring,
    ty: target
  } as Question;

  // Recreate the required fields based on the previous type
  switch (base.ty) {
    case QuestionType.Single:
    case QuestionType.Multiple:
      // Try and inherit answers from the alternative type
      if (
        question.ty === QuestionType.Single ||
        question.ty === QuestionType.Multiple
      ) {
        base.answers = question.answers;
      } else {
        base.answers = defaultAnswers();
      }

      break;
    case QuestionType.TrueFalse:
      base.answer = true;
      break;
    case QuestionType.Typer:
      base.answers = ["Example Answer"];
      base.ignore_case = true;
      break;
  }

  return base;
}

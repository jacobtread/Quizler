import { ScoreType } from "$lib/socket/models";
import { randomRange } from "./utils";

const messages = {
  Answered: [
    "Hmm I wonder if you got it right....",
    "It definitely had to be that one! 👌",
    "Yeah it was probably that one... 😬",
    "0_0 good luck i guess..",
    "Are you sure it was that one..?",
    "😅 Probbably right",
    "It gotta be that one!",
    "If you say so 🫠",
    "1 + 1 = 5?",
    "Was definitely that one! 😎",
    "🤫 don't tell them the answer",
    "Made it in time 😮‍💨",
    "Nice answer 🥳",
    "Mastermind 🧠"
  ],
  [ScoreType.Correct]: [
    "You did it!",
    "That one was right!",
    "Good job!",
    "Yup that was it!",
    "Nicely done 👍",
    "Brainy 🧠",
    "Smarty pants 🤓"
  ],
  [ScoreType.Incorrect]: [
    "Ooops..",
    "Yeah not that one... 🤨",
    "Better luck next time 🤧",
    "Noooo your other left",
    "Uh oh.... 😨",
    "Yeah nope.... ",
    "😤 Maybe next time"
  ],
  [ScoreType.Partial]: [
    "Almost there...",
    "Not quite there",
    "Just about...",
    "Yeah not that one... 🤨",
    "Better luck next time 🤧",
    "Noooo your other left",
    "Uh oh.... 😨",
    "Yeah nope.... ",
    "😤 Maybe next time"
  ]
};

/**
 * Returns a random message from the defined list of
 * messages above based on the specific type category
 * provided
 *
 * @param ty The type of message
 * @returns The random message
 */
export function getRandomMessage(ty: "Answered" | ScoreType) {
  const values: string[] = messages[ty];
  const message: string = values[randomRange(0, values.length - 1)];
  return message;
}

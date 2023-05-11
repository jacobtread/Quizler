import { z } from "zod";

export type SessionId = number;
export type GameToken = string;
export type TimerState = { total: number; elapsed: number };
export type Scores = Record<SessionId, number>;
export type ImageRef = string;
export type QuestionIndex = number;

// Request structure used for creating a quiz
export interface CreateRequest {
  name: string;
  text: string;
  timing: TimingConfig;
  questions: Question[];
}

// Response structure for a created quiz
export interface CreatedResponse {
  // UUID of the prepared game
  uuid: string;
}

export interface OtherPlayer {
  id: SessionId;
  name: string;
}

export interface GameConfig {
  name: string;
  text: string;
}

export interface TimingConfig {
  wait_time: number;
}

export const enum ServerError {
  MalformedMessage = "MalformedMessage",
  InvalidToken = "InvalidToken",
  UsernameTaken = "UsernameTaken",
  NotJoinable = "NotJoinable",
  UnknownPlayer = "UnknownPlayer",
  Unexpected = "Unexpected",
  InvalidPermission = "InvalidPermission",
  UnexpectedMessage = "UnexpectedMessage",
  InvalidAnswer = "InvalidAnswer"
}

export const errorText: Record<ServerError, string> = {
  [ServerError.MalformedMessage]: "Unknown client sent invalid message",
  [ServerError.InvalidToken]: "Invalid token provided",
  [ServerError.UsernameTaken]: "Username already in use",
  [ServerError.NotJoinable]: "Quiz is not joinable",
  [ServerError.UnknownPlayer]: "Target player not found",
  [ServerError.Unexpected]: "Unexpected error occurred",
  [ServerError.InvalidPermission]: "You don't have permission to do that",
  [ServerError.UnexpectedMessage]: "Client and server out of sync",
  [ServerError.InvalidAnswer]: "Invalid answer type"
};

export const enum GameState {
  Lobby = "Lobby",
  Starting = "Starting",
  AwaitingReady = "AwaitingReady",
  AwaitingAnswers = "AwaitingAnswers",
  Marked = "Marked",
  Finished = "Finished"
}

export const enum HostAction {
  Start = "Start",
  Cancel = "Cancel",
  Skip = "Skip",
  Reset = "Reset"
}

export const enum RemoveReason {
  RemovedByHost = "RemovedByHost",
  HostDisconnect = "HostDisconnect",
  LostConnection = "LostConnection",
  Disconnected = "Disconnected"
}

export const enum QuestionType {
  Single = "Single",
  Multiple = "Multiple"
}

const answerValueSchema = z.object({
  id: z.number(),
  value: z.string(),
  correct: z.boolean()
});

export type AnswerValue = z.infer<typeof answerValueSchema>;

export const questionSchema = z
  .object({
    id: z.number(),
    text: z.string(),
    image: z.string().uuid().nullable(),
    answer_time: z.number(),
    bonus_score_time: z.number(),
    scoring: z.object({
      min_score: z.number(),
      max_score: z.number(),
      bonus_score: z.number()
    })
  })
  .and(
    z.discriminatedUnion("ty", [
      // Single choice questions
      z.object({
        ty: z.literal(QuestionType.Single),
        answers: z.array(answerValueSchema)
      }),
      // Multiple choice questions
      z.object({
        ty: z.literal(QuestionType.Multiple),
        answers: z.array(answerValueSchema),
        min: z.number(),
        max: z.number()
      })
    ])
  );

export type Question = z.infer<typeof questionSchema>;

export const enum AnswerType {
  Single = "Single",
  Multiple = "Multiple"
}
export type Answer =
  | { ty: AnswerType.Single; answer: number }
  | { ty: AnswerType.Multiple; answers: number[] };

export const enum ScoreType {
  Correct = "Correct",
  Incorrect = "Incorrect",
  Partial = "Partial"
}

export type Score =
  | { ty: ScoreType.Correct; value: number }
  | {
      ty: ScoreType.Partial;
      count: number;
      total: number;
      value: number;
    }
  | { ty: ScoreType.Incorrect };

/* 
  CLIENT MESSAGES
*/

export const enum ClientMessage {
  Initialize = "Initialize",
  Connect = "Connect",
  Join = "Join",
  Ready = "Ready",
  HostAction = "HostAction",
  Answer = "Answer",
  Kick = "Kick"
}

export type ClientMessageSchema = {
  rid?: number;
} & (
  | { ty: ClientMessage.Initialize; uuid: string }
  | { ty: ClientMessage.Connect; token: GameToken }
  | { ty: ClientMessage.Join; name: string }
  | { ty: ClientMessage.Ready }
  | { ty: ClientMessage.HostAction; action: HostAction }
  | { ty: ClientMessage.Answer; answer: Answer }
  | { ty: ClientMessage.Kick; id: SessionId }
);

export type ClientMessageOf<T> = Extract<ClientMessageSchema, { ty: T }>;

/* 
  SERVER MESSAGES
*/

export const enum ServerMessage {
  Joined = "Joined",
  Ok = "Ok",
  OtherPlayer = "OtherPlayer",
  GameState = "GameState",
  TimeSync = "TimeSync",
  Question = "Question",
  Scores = "Scores",
  Score = "Score",
  Error = "Error",
  Kicked = "Kicked"
}

export type ServerMessageSchema = {
  rid?: number;
} & (
  | { ty: ServerMessage.Joined; id: number; token: string; config: GameConfig }
  | { ty: ServerMessage.OtherPlayer; id: number; name: string }
  | { ty: ServerMessage.GameState; state: GameState }
  | { ty: ServerMessage.TimeSync; total: number; elapsed: number }
  | { ty: ServerMessage.Question; question: Question }
  | { ty: ServerMessage.Scores; scores: Scores }
  | { ty: ServerMessage.Score; score: Score }
  | { ty: ServerMessage.Error; error: ServerError }
  | { ty: ServerMessage.Kicked; id: number; reason: RemoveReason }
  | { ty: ServerMessage.Ok }
);

export type ServerMessageOf<T> = Extract<ServerMessageSchema, { ty: T }>;

// Converts from client message to server message type
export type PairMessageType<T> = T extends
  | ClientMessage.Initialize
  | ClientMessage.Join
  ? ServerMessage.Joined
  : T extends
      | ClientMessage.Connect
      | ClientMessage.Ready
      | ClientMessage.HostAction
      | ClientMessage.Answer
      | ClientMessage.Kick
  ? ServerMessage.Ok
  : unknown;

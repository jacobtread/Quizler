import { z } from "zod";

export interface CreatedResponse {
  uuid: string;
}

export type SessionId = number;
export type GameToken = string;
export type TimerState = { total: number; elapsed: number };
export type Scores = Record<SessionId, number>;

export interface JoinedMessage {
  id: SessionId;
  token: GameToken;
  config: GameConfig;
}

export type OtherPlayerMessage = OtherPlayer;

export interface OtherPlayer {
  id: SessionId;
  name: string;
}

export interface GameStateMessage {
  state: GameState;
}

export type TimeSyncMessage = TimerState;

export interface QuestionMessage {
  question: Question;
}

export interface ScoresMessage {
  scores: Scores;
}

export interface ScoreMessage {
  score: Score;
}

export interface ErrorMessage {
  error: ServerError;
}

export interface KickedMessage {
  session_id: SessionId;
  reason: RemoveReason;
}

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

export type Message<T> = {
  ty: T;
  rid?: number | undefined;
} & ServerMessageBody<T>;

// Transforms the provided ServerMessage variant into the associated
// message content for that message
export type ServerMessageBody<T> = T extends ServerMessage.Joined
  ? JoinedMessage
  : T extends ServerMessage.OtherPlayer
  ? OtherPlayerMessage
  : T extends ServerMessage.GameState
  ? GameStateMessage
  : T extends ServerMessage.TimeSync
  ? TimeSyncMessage
  : T extends ServerMessage.Question
  ? QuestionMessage
  : T extends ServerMessage.Scores
  ? ScoresMessage
  : T extends ServerMessage.Score
  ? ScoreMessage
  : T extends ServerMessage.Error
  ? ErrorMessage
  : T extends ServerMessage.Kicked
  ? KickedMessage
  : T extends ServerMessage.Ok
  ? OkMessage
  : unknown;

export type OkMessage = Record<string, never>;

export interface InitializeMessage {
  uuid: string;
}

export interface ConnectMessage {
  token: GameToken;
}

export interface JoinMessage {
  name: string;
}

export interface HostActionMessage {
  action: HostAction;
}

export interface AnswerMessage {
  answer: Answer;
}

export interface KickMessage {
  id: SessionId;
}

export const enum ClientMessage {
  Initialize = "Initialize",
  Connect = "Connect",
  Join = "Join",
  Ready = "Ready",
  HostAction = "HostAction",
  Answer = "Answer",
  Kick = "Kick"
}

export type ClientMessageBody<T extends ClientMessage> =
  T extends ClientMessage.Initialize
    ? InitializeMessage
    : T extends ClientMessage.Connect
    ? ConnectMessage
    : T extends ClientMessage.Join
    ? JoinMessage
    : T extends ClientMessage.Ready
    ? Record<string, never>
    : T extends ClientMessage.HostAction
    ? HostActionMessage
    : T extends ClientMessage.Answer
    ? AnswerMessage
    : T extends ClientMessage.Kick
    ? KickMessage
    : unknown;

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

export interface UploadConfig {
  name: string;
  text: string;
  timing: TimingConfig;
  questions: Question[];
}

export interface GameConfig {
  name: string;
  text: string;
}

export interface TimingConfig {
  wait_time: number;
}

export const timingConfigSchema = z.object({
  wait_time: z.number()
});

export const enum GameState {
  Lobby = "Lobby",
  Starting = "Starting",
  AwaitingReady = "AwaitingReady",
  AwaitingAnswers = "AwaitingAnswers",
  Marked = "Marked",
  Finished = "Finished"
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

export const enum HostAction {
  Start = "Start",
  Cancel = "Cancel",
  Skip = "Skip"
}

export const enum RemoveReason {
  RemovedByHost = "RemovedByHost",
  HostDisconnect = "HostDisconnect",
  LostConnection = "LostConnection",
  Disconnected = "Disconnected"
}

export type ImageRef = string;
export type QuestionIndex = number;

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

const answerSchema = z.discriminatedUnion("ty", [
  // Single choice answer
  z.object({
    ty: z.literal(AnswerType.Single),
    answers: z.number()
  }),
  // Single choice answer
  z.object({
    ty: z.literal(AnswerType.Multiple),
    answers: z.array(z.number())
  })
]);

export type Answer = z.infer<typeof answerSchema>;

export const enum ScoreType {
  Correct = "Correct",
  Incorrect = "Incorrect",
  Partial = "Partial"
}

const scoreSchema = z.discriminatedUnion("ty", [
  z.object({
    ty: z.literal(ScoreType.Correct),
    value: z.number()
  }),
  z.object({
    ty: z.literal(ScoreType.Partial),
    value: z.object({
      count: z.number(),
      total: z.number(),
      socre: z.number()
    })
  }),
  z.object({
    ty: z.literal(ScoreType.Incorrect)
  })
]);

export type Score = z.infer<typeof scoreSchema>;

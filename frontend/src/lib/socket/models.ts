export interface CreatedResponse {
  uuid: string;
}

export type SessionId = number;
export type GameToken = string;

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
export type TimerState = { total: number; elapsed: number };

export type TimeSyncMessage = TimerState;

export interface QuestionMessage {
  question: Question;
}

export interface ScoresMessage {
  scores: Record<SessionId, number>;
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

export const enum ClientMessageType {
  Initialize = "Initialize",
  Connect = "Connect",
  Join = "Join",
  Ready = "Ready",
  HostAction = "HostAction",
  Answer = "Answer",
  Kick = "Kick"
}

export type ClientMessageBody<T> = T extends ClientMessageType.Initialize
  ? InitializeMessage
  : T extends ClientMessageType.Connect
  ? ConnectMessage
  : T extends ClientMessageType.Join
  ? JoinMessage
  : T extends ClientMessageType.Ready
  ? Record<string, never>
  : T extends ClientMessageType.HostAction
  ? HostActionMessage
  : T extends ClientMessageType.Answer
  ? AnswerMessage
  : T extends ClientMessageType.Kick
  ? KickMessage
  : unknown;

export type PairMessageType<T> = T extends
  | ClientMessageType.Initialize
  | ClientMessageType.Join
  ? ServerMessage.Joined
  : T extends
      | ClientMessageType.Connect
      | ClientMessageType.Ready
      | ClientMessageType.HostAction
      | ClientMessageType.Answer
      | ClientMessageType.Kick
  ? ServerMessage.Ok
  : unknown;

export interface UploadConfig {
  basic: BasicConfig;
  timing: TimingConfig;
  questions: Question[];
}

export interface GameConfig {
  basic: BasicConfig;
}

export interface BasicConfig {
  name: string;
  text: string;
}

export interface TimingConfig {
  wait_time: number;
  bonus_score_time: number;
}

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
  AlreadyAnswered = "AlreadyAnswered",
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

export interface AnswerValue {
  id: number;
  value: string;
  correct: boolean;
}

interface SingleQuestionData {
  answers: AnswerValue[];
}

export interface MultipleQuestionData {
  answers: AnswerValue[];
  min: number;
  max: number;
}

export const enum QuestionDataType {
  Single = "Single",
  Multiple = "Multiple"
}

export interface QuestionBase {
  // Only present while created the question as a unique key
  id: number;
  text: string;
  image: ImageRef | null;
  answer_time: number;
  scoring: Scoring;
}

export type Question = QuestionBase &
  (
    | ({ ty: QuestionDataType.Single } & SingleQuestionData)
    | ({ ty: QuestionDataType.Multiple } & MultipleQuestionData)
  );

export interface Scoring {
  min_score: number;
  max_score: number;
  bonus_score: number;
}

export const enum AnswerType {
  Single = "Single",
  Multiple = "Multiple"
}

interface SingleAnswer {
  answer: QuestionIndex;
}

export interface MultipleAnswer {
  answers: QuestionIndex[];
}

export type Answer =
  | ({ ty: AnswerType.Single } & SingleAnswer)
  | ({ ty: AnswerType.Multiple } & MultipleAnswer);

export const enum ScoreType {
  Correct = "Correct",
  Incorrect = "Incorrect",
  Partial = "Partial"
}

export type Score =
  | { ty: ScoreType.Correct; value: number }
  | { ty: ScoreType.Partial; value: number }
  | { ty: ScoreType.Incorrect };

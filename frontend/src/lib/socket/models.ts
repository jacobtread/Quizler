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

export interface OtherPlayerMessage {
  id: SessionId;
  name: string;
}

export interface GameStateMessage {
  state: GameState;
}

export interface TimeSyncMessage {
  total: number;
  elapsed: number;
}

export interface QuestionMessage {
  question: Question;
}

export interface ScoresMessage {
  scores: Record<SessionId, number>;
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
  OtherPlayer = "OtherPlayer",
  GameState = "GameState",
  TimeSync = "TimeSync",
  Question = "Question",
  Scores = "Scores",
  Error = "Error",
  Kicked = "Kicked",
}

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
  : T extends ServerMessage.Error
  ? ErrorMessage
  : T extends ServerMessage.Kicked
  ? KickedMessage
  : unknown;

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
  Kick = "Kick",
}

export type ClientMessage =
  | ({ ty: ClientMessageType.Initialize } & InitializeMessage)
  | ({ ty: ClientMessageType.Connect } & ConnectMessage)
  | ({ ty: ClientMessageType.Join } & JoinMessage)
  | { ty: ClientMessageType.Ready }
  | ({ ty: ClientMessageType.HostAction } & HostActionMessage)
  | ({ ty: ClientMessageType.Answer } & AnswerMessage)
  | ({ ty: ClientMessageType.Kick } & KickMessage);

export interface GameConfig {
  basic: BasicConfig;
}

export interface BasicConfig {
  name: string;
  text: string;
}

export const enum GameState {
  Lobby = 0x0,
  Starting = 0x1,
  AwaitingReady = 0x2,
  AwaitingAnswers = 0x3,
  Marked = 0x4,
  Finished = 0x5,
}

export const enum ServerError {
  MalformedMessage = 0x0,
  InvalidToken = 0x1,
  UsernameTaken = 0x2,
  NotJoinable = 0x3,
  UnknownPlayer = 0x4,
  Unexpected = 0x5,
  InvalidPermission = 0x6,
  UnexpectedMessage = 0x7,
  AlreadyAnswered = 0x8,
  InvalidAnswer = 0x9,
}

export const enum HostAction {
  Start = 0x1,
  Cancel = 0x2,
  Skip = 0x3,
}

export const enum RemoveReason {
  RemovedByHost = 0x1,
  HostDisconnect = 0x2,
  LostConnection = 0x3,
  Disconnected = 0x4,
}

type ImageRef = string;
type QuestionIndex = number;

interface SingleQuestionData {
  values: string[];
}

export interface MultipleQuestionData {
  values: string[];
  min: number | null;
  max: number | null;
}

export interface ClickableImageQuestionData {
  imag: ImageRef;
}

export const enum QuestionDataType {
  Single = "Single",
  Multiple = "Multiple",
  ClickableImage = "ClickableImage",
}

export interface QuestionBase {
  title: string;
  text: string;
  image: ImageRef | null;
  answer_time: number;
  scoring: Scoring;
}

export type Question = QuestionBase &
  (
    | ({ ty: QuestionDataType.Single } & SingleQuestionData)
    | ({ ty: QuestionDataType.Multiple } & MultipleQuestionData)
    | ({ ty: QuestionDataType.ClickableImage } & ClickableImageQuestionData)
  );

export interface Scoring {
  min_score: number;
  max_score: number;
  bonus_score: number;
}

export const enum AnswerType {
  Single = "Single",
  Multiple = "Multiple",
  ClickableImage = "ClickableImage",
}

interface SingleAnswer {
  answer: QuestionIndex;
}

export interface MultipleAnswer {
  answers: QuestionIndex[];
}

export interface ClickableImageAnswer {
  answer: [number, number];
}

export type Answer =
  | ({ ty: AnswerType.Single } & SingleAnswer)
  | ({ ty: AnswerType.Multiple } & MultipleAnswer)
  | ({ ty: AnswerType.ClickableImage } & ClickableImageAnswer);

import { z } from "zod";

// Represents a unique ID of a session
export type SessionId = number;

// Represents a 5 character game token (e.g EAU32)
export type GameToken = string;

// State of the quiz timer
export type TimerState = { total: number; elapsed: number };

// Mapping between IDs and the scores
export type Scores = Record<SessionId, number>;

// UUID to an image on the server
export type ImageRef = string;

// Snapshot of the game state at completion
// to keep around the scores and players
export interface GameSummary {
  /// Summary for each of the players in the game
  players: PlayerSummary[];
}

// Extended player data to include score
export type PlayerSummary = PlayerData & { score: number };

// Response structure for a created quiz
export interface CreatedResponse {
  // UUID of the prepared game
  uuid: string;
}

// Basic player data
export interface PlayerData {
  // The ID of the player
  id: SessionId;
  // The name of the player
  name: string;
}

// Quiz configuration
export interface GameConfig {
  // Name of the quiz
  name: string;
  // Text description of the quiz
  text: string;
}

// Server error types
export const enum ServerError {
  MalformedMessage = "MalformedMessage",
  InvalidToken = "InvalidToken",
  UsernameTaken = "UsernameTaken",
  InappropriateName = "InappropriateName",
  NotJoinable = "NotJoinable",
  CapacityReached = "CapacityReached",
  UnknownPlayer = "UnknownPlayer",
  Unexpected = "Unexpected",
  InvalidPermission = "InvalidPermission",
  UnexpectedMessage = "UnexpectedMessage",
  InvalidAnswer = "InvalidAnswer"
}

// Messages for different server errors
export const errorText: Record<ServerError, string> = {
  [ServerError.MalformedMessage]: "Unknown client sent invalid message",
  [ServerError.InvalidToken]: "Invalid token provided",
  [ServerError.UsernameTaken]: "Username already in use",
  [ServerError.InappropriateName]:
    "That name is not allowed/inappropriate choose another name",
  [ServerError.NotJoinable]: "Quiz is not joinable",
  [ServerError.CapacityReached]: "Quiz is full",
  [ServerError.UnknownPlayer]: "Target player not found",
  [ServerError.Unexpected]: "Unexpected error occurred",
  [ServerError.InvalidPermission]: "You don't have permission to do that",
  [ServerError.UnexpectedMessage]: "Client and server out of sync",
  [ServerError.InvalidAnswer]: "Invalid answer type"
};

// Name filtering modes
export const enum NameFiltering {
  // Don't filter names anything goes
  None = "None",
  // Only stop the more severe names
  Low = "Low",
  // Stop anything thats above mild
  Medium = "Medium",
  // Filter out any names that might be inappropriate
  High = "High"
}

// Possible game states
export const enum GameState {
  Lobby = "Lobby",
  Starting = "Starting",
  AwaitingReady = "AwaitingReady",
  AwaitingAnswers = "AwaitingAnswers",
  Marked = "Marked",
  Finished = "Finished"
}

// Actions that hosts can send to the server
export const enum HostAction {
  Start = "Start",
  Cancel = "Cancel",
  Skip = "Skip",
  Next = "Next",
  Reset = "Reset"
}

// Different remove reasons
export const enum RemoveReason {
  RemovedByHost = "RemovedByHost",
  HostDisconnect = "HostDisconnect",
  LostConnection = "LostConnection",
  Disconnected = "Disconnected"
}

// Messages for different removal reasons
export const removeReasonText: Record<RemoveReason, string> = {
  [RemoveReason.RemovedByHost]: "Removed by host",
  [RemoveReason.HostDisconnect]: "Quiz host left",
  [RemoveReason.LostConnection]: "Connection lost",
  [RemoveReason.Disconnected]: "Disconnected"
};

// Question types
export const enum QuestionType {
  Single = "Single",
  Multiple = "Multiple"
}

// Schema for question answers
const answerValueSchema = z.object({
  id: z.number(),
  value: z.string(),
  correct: z.boolean()
});

// Answer value type inferred from its schema
export type AnswerValue = z.infer<typeof answerValueSchema>;

export const enum ImageFit {
  Contain = "Contain",
  Cover = "Cover",
  Width = "Width",
  Height = "Height"
}

// Schema for questions
export const questionSchema = z
  .object({
    id: z.number(),
    text: z.string(),
    image: z
      .object({
        uuid: z.string().uuid(),
        fit: z.enum(["Contain", "Cover", "Width", "Height"])
      })
      .nullable(),
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

// Question type inferred from its schema
export type Question = z.infer<typeof questionSchema>;

// Different answer types
export const enum AnswerType {
  Single = "Single",
  Multiple = "Multiple"
}

// Answer schemas for each different type
export type Answer =
  | { ty: AnswerType.Single; answer: number }
  | { ty: AnswerType.Multiple; answers: number[] };

// Different score types
export const enum ScoreType {
  Correct = "Correct",
  Incorrect = "Incorrect",
  Partial = "Partial"
}

// Score schemas for each different type
export type Score =
  | { ty: ScoreType.Correct; value: number }
  | {
      ty: ScoreType.Partial;
      count: number;
      total: number;
      value: number;
    }
  | { ty: ScoreType.Incorrect };

// Client message types
export const enum ClientMessage {
  Initialize = "Initialize",
  Connect = "Connect",
  Join = "Join",
  Ready = "Ready",
  HostAction = "HostAction",
  Answer = "Answer",
  Kick = "Kick"
}

// Client message schema based on each message type
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

// Client message type extractor
export type ClientMessageOf<T> = Extract<ClientMessageSchema, { ty: T }>;

// Server message types
export const enum ServerMessage {
  Joined = "Joined",
  Ok = "Ok",
  PlayerData = "PlayerData",
  GameState = "GameState",
  TimeSync = "TimeSync",
  Question = "Question",
  Scores = "Scores",
  Score = "Score",
  Error = "Error",
  Kicked = "Kicked"
}

// Server message schema based on each message type
export type ServerMessageSchema = {
  rid?: number;
} & (
  | { ty: ServerMessage.Joined; id: number; token: string; config: GameConfig }
  | { ty: ServerMessage.PlayerData; id: number; name: string }
  | { ty: ServerMessage.GameState; state: GameState }
  | { ty: ServerMessage.TimeSync; total: number; elapsed: number }
  | { ty: ServerMessage.Question; question: Question }
  | { ty: ServerMessage.Scores; scores: Scores }
  | { ty: ServerMessage.Score; score: Score }
  | { ty: ServerMessage.Error; error: ServerError }
  | { ty: ServerMessage.Kicked; id: number; reason: RemoveReason }
  | { ty: ServerMessage.Ok }
);

// Server message type extractor
export type ServerMessageOf<T> = Extract<ServerMessageSchema, { ty: T }>;

// Mapping between client messages and the server message type
export type MessagePairs =
  | { left: ClientMessage.Initialize; right: ServerMessage.Joined }
  | { left: ClientMessage.Join; right: ServerMessage.Joined }
  | { left: ClientMessage.Connect; right: ServerMessage.Ok }
  | { left: ClientMessage.Ready; right: ServerMessage.Ok }
  | { left: ClientMessage.HostAction; right: ServerMessage.Ok }
  | { left: ClientMessage.Answer; right: ServerMessage.Ok }
  | { left: ClientMessage.Kick; right: ServerMessage.Ok };

// Converts from client message to server message type
export type ServerResponseOf<T> = Extract<MessagePairs, { left: T }>["right"];

// Response message type from the client message
export type ResponseMessage<T> = ServerMessageOf<ServerResponseOf<T>>;

import { writable } from "svelte/store";
import { DEBUG } from "../constants";
import {
  ServerMessage,
  type JoinedMessage,
  type OtherPlayerMessage,
  type GameStateMessage,
  type TimeSyncMessage,
  type QuestionMessage,
  type ScoresMessage,
  type ErrorMessage,
  type KickedMessage,
  type ServerMessageBody,
  type ClientMessage,
} from "./models";

type MessageHandler<T> = (msg: T) => void;
type MessageHandlers = {
  [T in ServerMessage]: MessageHandler<ServerMessageBody<T>>;
};

// Reference to the socket
let socket: WebSocket = createSocket();

// Map of the message types to their handlers
const messageHandlers: MessageHandlers = {
  [ServerMessage.Joined]: onJoined,
  [ServerMessage.OtherPlayer]: onOtherPlayer,
  [ServerMessage.GameState]: onGameState,
  [ServerMessage.TimeSync]: onTimeSync,
  [ServerMessage.Question]: onQuestion,
  [ServerMessage.Scores]: onScores,
  [ServerMessage.Error]: onError,
  [ServerMessage.Kicked]: onKicked,
};

// Socket readiness state
export const socketReady = writable<boolean>(false);

/**
 * Creates a new socket connection assigning
 * the event handlers for the different events
 */
function createSocket(): WebSocket {
  // Create the socket
  const ws = new WebSocket(getSocketURL());

  // Assign the message handler
  ws.onmessage = onMessage;

  // Handle open events
  ws.onopen = () => {
    // Handle the WebSocket connection becoming OPEN
    if (ws.readyState == WebSocket.OPEN) {
      socketReady.set(true);
    }
  };

  // Handle close events
  ws.onclose = (event: CloseEvent) => {
    // Handle the socket becoming unavailable
    socketReady.set(false);
    console.warn("WebSocket connetion closed", event);

    // Reconnect the socket
    socket = createSocket();
  };

  // Handle error events
  ws.onerror = (event: Event) => {
    // Handle the socket becoming unavailable
    socketReady.set(false);
    console.error("WebSocket error", event);

    // Reconnect the socket
    socket = createSocket();
  };

  return ws;
}

/**
 * Obtains a URL to the endpoint for connecting the WebSocket.
 * For debug mode this is a constant value otherwise the website
 * origin is used
 *
 * @returns The URL that the WebSocket should use
 */
function getSocketURL(): URL {
  const SOCKET_ENDPOINT: string = "/api/quiz/socket";

  let host = DEBUG
    ? "ws://localhost"
    : window.location.origin.replace(/^https/, "wss").replace(/^http/, "ws");

  return new URL(SOCKET_ENDPOINT, host);
}

/**
 * Sends the provided message to the server through
 * the socket
 *
 * @param msg
 */
export function sendMessage(msg: ClientMessage) {
  console.debug("Sending message to server", msg);

  const data = JSON.stringify(msg);
  try {
    socket.send(data);
  } catch (e) {
    console.error("Failed to send message", e);
  }
}

/**
 * Event handler for parsing, and handling messages
 * recieved from the server
 *
 * @param data The message data from the event
 */
function onMessage<T extends ServerMessage>({ data }: MessageEvent) {
  // Parse the message
  const msg: { ty: T | undefined } & ServerMessageBody<T> = JSON.parse(data);

  // Ensure the message type is specified
  if (msg.ty === undefined) {
    console.error("Packet missing message type", data);
    return;
  }

  // Find the handler for the message
  const handler = messageHandlers[msg.ty];
  if (handler === undefined) {
    console.error("Handler not defined for packet type", msg.ty);
    return;
  }

  // Call the handler with the mesasge
  handler.call(socket, msg);
}

function onJoined(msg: JoinedMessage) {
  console.debug("Joined message", msg);
}

function onOtherPlayer(msg: OtherPlayerMessage) {
  console.debug("Other player message", msg);
}

function onGameState(msg: GameStateMessage) {
  console.debug("Game state message", msg);
}

function onTimeSync(msg: TimeSyncMessage) {
  console.debug("Time sync message", msg);
}

function onQuestion(msg: QuestionMessage) {
  console.debug("Question message", msg);
}

function onScores(msg: ScoresMessage) {
  console.debug("Score message", msg);
}

function onError(msg: ErrorMessage) {
  console.error("Server error", msg.error);
}

function onKicked(msg: KickedMessage) {
  console.debug("Kick message", msg);
}

import { writable, type Unsubscriber } from "svelte/store";
import { DEBUG } from "../constants";
import {
  ServerMessage,
  type OtherPlayerMessage,
  type GameStateMessage,
  type TimeSyncMessage,
  type QuestionMessage,
  type ScoresMessage,
  type ErrorMessage,
  type KickedMessage,
  type ServerMessageBody,
  type ClientMessageBody,
  type PairMessageType
} from "./models";
import { clearGame, setOtherPlayer } from "../game";
import { AppState, appState } from "../state";

type MessageHandler<T> = (msg: T) => void;
type MessageHandlers = {
  [T in ServerMessage]?: MessageHandler<ServerMessageBody<T>>;
};
type RequestHandle<T> = (msg: T) => void;

// The next request ID to use
let requestHandle: number = 0;
// Handlers from requests awaiting responses
let requestHandles: Record<number, RequestHandle<any> | undefined> = {};

// Reference to the socket
let socket: WebSocket = createSocket();

// Global message handlers for packet notifications
const messageHandlers: MessageHandlers = {
  [ServerMessage.OtherPlayer]: onOtherPlayer,
  [ServerMessage.GameState]: onGameState,
  [ServerMessage.TimeSync]: onTimeSync,
  [ServerMessage.Question]: onQuestion,
  [ServerMessage.Scores]: onScores,
  [ServerMessage.Error]: onError,
  [ServerMessage.Kicked]: onKicked
};

// Socket readiness state
export const socketReady = writable<boolean>(false);

// State determining if we are the game host
export const gameHost = writable<boolean>(false);

/**
 * Creates a promise that subscribes to when
 * the socket connection is ready to be used
 *
 * @returns The ready promise
 */
export function getSocketReady(): Promise<void> {
  // Unsubscribe callback for cleaning up subscription
  let unsub: Unsubscriber | undefined;

  return (
    new Promise<void>((resolve) => {
      unsub = socketReady.subscribe((value) => {
        if (value) {
          resolve();
        }
      });
    })
      // Remove subscription
      .finally(unsub)
  );
}

/**
 * Creates a new socket connection assigning
 * the event handlers for the different events
 */
function createSocket(): WebSocket {
  // Reset request counter
  requestHandle = 0;
  requestHandles = {};

  const socketUrl = getSocketURL();

  console.debug("Connecting to socket server " + socketUrl);

  // Create the socket
  const ws = new WebSocket(socketUrl);

  // Assign the message handler
  ws.onmessage = onMessage;

  // Handle open events
  ws.onopen = () => {
    // Handle the WebSocket connection becoming OPEN
    if (ws.readyState == WebSocket.OPEN) {
      console.debug("Connected to socket");
      socketReady.set(true);
    } else {
      console.log(ws.readyState);
    }
  };

  // Handle close events
  ws.onclose = (event: CloseEvent) => {
    // Handle the socket becoming unavailable
    console.error("WebSocket connetion closed", event);

    // Update lost connection states
    onDisconnected();

    queueReconnect();
  };

  // Handle error events
  ws.onerror = (event: Event) => {
    // Handle the socket becoming unavailable
    console.error("WebSocket error", event);
  };

  return ws;
}

function onDisconnected() {
  clearGame();
  appState.set(AppState.Home);
}

/**
 * Handles clearing the socket ready state and setting
 * a timeout for when to reconnect
 */
function queueReconnect() {
  console.debug("Socket connection lost (Reconnecting in 1000ms)");

  socketReady.set(false);

  // Don't immediately try to reconnect
  setTimeout(() => {
    // Try reconnect the socket
    socket = createSocket();
  }, 1000);
}

/**
 * Obtains a URL to the endpoint for connecting the WebSocket.
 * For debug mode this is a constant value otherwise the website
 * origin is used
 *
 * @returns The URL that the WebSocket should use
 */
function getSocketURL(): URL {
  const SOCKET_ENDPOINT = "/api/quiz/socket";

  const host = DEBUG
    ? "ws://localhost"
    : window.location.origin.replace(/^https/, "wss").replace(/^http/, "ws");

  return new URL(SOCKET_ENDPOINT, host);
}

type ResponseOrError<T> =
  | ({ ty: T } & ServerMessageBody<T>)
  | ({ ty: ServerMessage.Error } & ErrorMessage);

/**
 * Sends the provided message to the server through
 * the socket
 *
 * @param msg
 */
export function sendMessage<T>(
  msg: { ty: T; rid?: number } & ClientMessageBody<T>
): Promise<ResponseOrError<PairMessageType<T>>> {
  return new Promise((resolve, reject) => {
    console.debug("Sending message to server", msg);

    msg.rid = requestHandle;
    requestHandle++;

    requestHandles[msg.rid] = resolve;

    const data = JSON.stringify(msg);
    try {
      socket.send(data);
    } catch (e) {
      console.error("Failed to send message", e);
      delete requestHandles[msg.rid];
      reject(e);
    }
  });
}

/**
 * Event handler for parsing, and handling messages
 * recieved from the server
 *
 * @param data The message data from the event
 */
function onMessage<T extends ServerMessage>({ data }: MessageEvent) {
  // Parse the message
  const msg: {
    ty: T | undefined;
    rid: number | undefined;
  } & ServerMessageBody<T> = JSON.parse(data);

  // Ensure the message type is specified
  if (msg.ty === undefined) {
    console.error("Packet missing message type", data);
    return;
  }

  const rid = msg.rid;
  if (rid !== undefined) {
    const handle = requestHandles[rid];
    if (handle !== undefined) {
      handle(msg);
    } else {
      console.error(`Missing return handle ${rid} for message`, msg);
    }
    return;
  }

  // Find the handler for the message
  const handler = messageHandlers[msg.ty];
  if (handler === undefined) {
    console.error("Handler not defined for packet type", msg.ty);
    return;
  }

  // Call the handler with the mesasge
  handler(msg);
}

function onOtherPlayer(msg: OtherPlayerMessage) {
  console.debug("Other player message", msg);
  setOtherPlayer(msg.id, msg.name);
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

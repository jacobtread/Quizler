import { writable, type Unsubscriber } from "svelte/store";
import { DEBUG } from "$lib/constants";
import {
  ServerMessage,
  type ErrorMessage,
  type ServerMessageBody,
  type ClientMessageBody,
  type PairMessageType,
  type Message,
  ClientMessage
} from "./models";
import { setHome } from "$stores/state";
import { onDestroy, onMount } from "svelte";

type MessageHandler<T> = (msg: T) => void;
type MessageHandlers = {
  [T in ServerMessage]?: MessageHandler<ServerMessageBody<T>>;
};

// The next request ID to use
let requestHandle: number = 0;
// Handlers from requests awaiting responses
let requestHandles: Record<number, MessageHandler<unknown> | undefined> = {};
// Queue of messages that haven't yet been handled
let messageQueue: Message<ServerMessage>[] = [];

// Reference to the socket
let socket: WebSocket = createSocket();

// Currently set message handlers for handling messages
export const messageHandlers: MessageHandlers = {};

// Socket readiness state
export const socketReady = writable<boolean>(false);

/**
 * Subscribes to a specific type of message within a svelte
 * component context.
 *
 * Handles removing the handler on the component destroy
 *
 * @param ty
 * @param handler
 */
export function setHandler<T extends ServerMessage>(
  ty: T,
  handler: MessageHandler<ServerMessageBody<T>>
) {
  // Append the handler on mount
  onMount(() => {
    // @ts-ignore
    messageHandlers[ty] = handler;
    console.log("Added handler for", ty);

    // Process matching queued messages
    messageQueue = messageQueue.filter((msg) => {
      if (msg.ty === ty) {
        // Handle messages that match the handler type
        handler(msg as Message<T>);
        return false;
      } else {
        return true;
      }
    });
  });

  // Remove the handler on unMount
  onDestroy(() => {
    delete messageHandlers[ty];
  });
}

/**
 * Creates a promise that subscribes to when
 * the socket connection is ready to be used
 *
 * @returns The ready promise
 */
export function ready(): Promise<void> {
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

function clearMessageQueue() {
  for (const msg of messageQueue) {
    console.warn("Message was not handled", msg);
  }

  messageQueue = [];
}

/**
 * Creates a new socket connection assigning
 * the event handlers for the different events
 */
function createSocket(): WebSocket {
  // Reset request counter
  requestHandle = 0;
  requestHandles = {};
  clearMessageQueue();

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
  };

  // Handle error events
  ws.onerror = (event: Event) => {
    // Handle the socket becoming unavailable
    console.error("WebSocket error", event);
  };

  return ws;
}

function onDisconnected() {
  // Return to the home screen
  setHome();

  // Attempt to reconnect
  queueReconnect();
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
export function send<T extends ClientMessage>(
  ty: T,
  body: ClientMessageBody<T>
): Promise<ResponseOrError<PairMessageType<T>>> {
  return new Promise((resolve, reject) => {
    console.debug("Sending message to server", ty, body);

    const msg = {
      rid: requestHandle,
      ty,
      ...body
    };
    requestHandle++;
    requestHandles[msg.rid] = resolve as MessageHandler<unknown>;

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
  const msg: Message<T> = JSON.parse(data);

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
  if (handler !== undefined) {
    // Call the handler with the mesasge
    handler(msg);
    return;
  }

  // Push the message to the queue
  messageQueue.push(msg);
}

import { writable } from "svelte/store";
import { onDestroy, onMount } from "svelte";
import {
  ServerEvent,
  type ServerEventOf as ServerEventOf,
  ServerError,
  ServerResponse,
  ClientMessage,
  type ClientMessageOf,
  type ServerResponseOf,
  type ServerMessage
} from "$api/models";
import { setHome } from "$stores/state";
import { getServerURL } from "$api/http";

// Handler function that expects a specific server message type
type ServerEventHandler<Type> = (msg: ServerEventOf<Type>) => void;
// Handler function that expects a specific server message type
type ServerResponseHandler<Type> = (msg: ServerResponseOf<Type>) => void;

// Type of a handler for handling a message response
interface RequestHandler<T> {
  // Handler resolve callback with the response
  resolve: ServerResponseHandler<T>;
  // Handler reject callback with a server error
  reject(err: ServerError): void;
}

// The next request ID to use
let requestHandle: number = 0;
// Handlers from requests awaiting responses
let requestHandles: Partial<Record<number, RequestHandler<unknown>>> = {};
// Queue of messages that haven't yet been handled
let messageQueue: ServerMessage[] = [];

// Reference to the socket
let socket: WebSocket = createSocket();

// Currently set message handlers for handling messages
const messageHandlers: Partial<
  Record<ServerEvent, ServerEventHandler<unknown>>
> = {};

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
export function setHandler<T extends ServerEvent>(
  ty: T,
  handler: ServerEventHandler<T>
) {
  // Append the handler on mount
  onMount(() => {
    // @ts-ignore
    messageHandlers[ty] = handler;
    console.debug("Added handler for", ty);

    // Process matching queued messages
    messageQueue = messageQueue.filter((msg) => {
      if (msg.ty === ty) {
        // Handle messages that match the handler type
        handler(msg as ServerEventOf<T>);
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
 * Logs out all the unhandled messages in the queue
 * then clears the queue
 */
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
  const socketUrl = getServerURL("/api/quiz/socket");
  // Replace the url protocol with the correct socket protocol
  socketUrl.protocol = socketUrl.protocol.includes("https") ? "wss" : "ws";

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
  // Reset the state
  requestHandle = 0;
  requestHandles = {};
  clearMessageQueue();

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
 * Sends the provided message to the server through
 * the socket
 *
 * @throws The server error or socket error
 * @param msg
 */
export function send<T extends ClientMessage>(
  msg: ClientMessageOf<T>
): Promise<ServerResponseOf<T>> {
  return new Promise((resolve, reject) => {
    console.debug("Sending message to server", msg);

    // Set the return ID and increase it for the next request
    msg.rid = requestHandle++;

    requestHandles[msg.rid] = {
      resolve: resolve as ServerResponseHandler<unknown>,
      reject
    };

    const data = JSON.stringify(msg);
    try {
      socket.send(data);
    } catch (e) {
      console.error("Failed to send message", e);
      delete requestHandles[msg.rid];
      reject({
        ty: ServerResponse.Error,
        error: "Unexpected"
      });
    }
  });
}

/**
 * Event handler for parsing, and handling messages
 * recieved from the server
 *
 * @param data The message data from the event
 */
function onMessage({ data }: MessageEvent) {
  // Parse the message
  const msg: ServerMessage = JSON.parse(data);

  // Ensure the message type is specified
  if (msg.ty === undefined) {
    console.error("Packet missing message type", data);
    return;
  }

  // Handle messages with return IDs
  const rid = msg.rid;
  if (rid !== undefined) {
    const handle = requestHandles[rid];
    if (handle === undefined) {
      console.error(`Missing return handle ${rid} for message`, msg);
      return;
    }

    // Compare the message type
    if (msg.ty === ServerResponse.Error) {
      // If the types didn't match it must've been an error
      handle.reject(msg.error);
    } else {
      // Reoslve with the correct result
      handle.resolve(msg as ServerResponseOf<unknown>);
    }
  }

  // Find the handler for the message
  const handler = messageHandlers[msg.ty as ServerEvent] as
    | ServerEventHandler<typeof msg.ty>
    | undefined;
  if (handler !== undefined) {
    // Call the handler with the mesasge
    handler(msg as ServerEventOf<unknown>);
    return;
  }

  // Push the message to the queue
  messageQueue.push(msg);
}

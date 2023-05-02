import { writable } from "svelte/store";
import { DEBUG } from "../constants";
import { ServerMessageType, type JoinedMessage, type OtherPlayerMessage, type GameStateMessage, type TimeSyncMessage, type QuestionMessage, type ScoresMessage, type ErrorMessage, type KickedMessage, type ServerMessage, type ClientMessage } from "./models";


function getSocketURL(): URL {
    const SOCKET_ENDPOINT: string = "/api/quiz/socket";

    if (DEBUG) {
        return new URL(SOCKET_ENDPOINT, "ws://localhost");
    }

    let host = window.location.origin
        .replace(/^https/, 'wss')
        .replace(/^http/, 'ws')

    return new URL(SOCKET_ENDPOINT, host);
}

const socketStore = writable<QuizSocket | null>(null);



type PacketHandlers = {
    [T in ServerMessageType]: (msg: ServerMessage<T>) => void
}





// WebSocket wrapper that establishes and handles a connection
// to the Quiz logic
class QuizSocket {
    // The WebSocket connection
    private ws: WebSocket;

    // Defines the respective functions that will handle each message
    // type from the server
    private handlers: PacketHandlers = {
        [ServerMessageType.Joined]: this.handleJoined.bind(this),
        [ServerMessageType.OtherPlayer]: this.handleOtherPlayer.bind(this),
        [ServerMessageType.GameState]: this.handleGameState.bind(this),
        [ServerMessageType.TimeSync]: this.handleTimeSync.bind(this),
        [ServerMessageType.Question]: this.handleQuestion.bind(this),
        [ServerMessageType.Scores]: this.handleScores.bind(this),
        [ServerMessageType.Error]: this.handleError.bind(this),
        [ServerMessageType.Kicked]: this.handleKicked.bind(this),
    }

    constructor() {
        const ws = new WebSocket(getSocketURL())
        ws.onmessage = this.onMessage.bind(this);
        this.ws = ws;
    }

    send(msg: ClientMessage) {
        const data = JSON.stringify(msg);
        this.ws.send(data);
    }

    onMessage<T extends ServerMessageType>(event: MessageEvent) {
        console.log(this);
        console.log(event);


        const data: string = event.data;
        const msg: { ty: T } & ServerMessage<T> = JSON.parse(data);

        console.log(msg);

        const handler = this.handlers[msg.ty];

        if (handler) {
            handler(msg);
        }

    }

    handleJoined(msg: JoinedMessage) {

    }

    handleOtherPlayer(msg: OtherPlayerMessage) {

    }

    handleGameState(msg: GameStateMessage) {

    }

    handleTimeSync(msg: TimeSyncMessage) {

    }

    handleQuestion(msg: QuestionMessage) {

    }

    handleScores(msg: ScoresMessage) {

    }

    handleError(msg: ErrorMessage) {

    }

    handleKicked(msg: KickedMessage) {

    }
}
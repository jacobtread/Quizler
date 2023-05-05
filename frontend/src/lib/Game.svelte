<script lang="ts">
  import { setMessageHandler } from "./socket";
  import {
    ServerMessage,
    type OtherPlayerMessage,
    type GameStateMessage,
    type TimeSyncMessage,
    type QuestionMessage,
    type ScoresMessage,
    type ErrorMessage,
    type KickedMessage,
    type OtherPlayer,
    GameState
  } from "./socket/models";
  import type { GameData } from "./state";

  export let gameData: GameData;

  // Hook the handlers for the different message types
  setMessageHandler(ServerMessage.OtherPlayer, onOtherPlayer);
  setMessageHandler(ServerMessage.GameState, onGameState);
  setMessageHandler(ServerMessage.TimeSync, onTimeSync);
  setMessageHandler(ServerMessage.Question, onQuestion);
  setMessageHandler(ServerMessage.Scores, onScores);
  setMessageHandler(ServerMessage.Error, onError);
  setMessageHandler(ServerMessage.Kicked, onKicked);

  let players: OtherPlayer[] = [];
  let gameState: GameState = GameState.Lobby;

  function onOtherPlayer(msg: OtherPlayerMessage) {
    console.debug("Other player message", msg);
    // Add to the players list
    players.push(msg);

    players = players;
  }

  function onGameState(msg: GameStateMessage) {
    console.debug("Game state message", msg);
    gameState = msg.state;
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
    // Remove from the players list
    players = players.filter((player) => player.id === msg.session_id);
  }
</script>

<h1>Game</h1>
<p>Host: {gameData.host}</p>
<p>{gameState}</p>
<p>{gameData.token}</p>

<ul>
  {#each players as player}
    <li>{player.id}: {player.name}</li>
  {/each}
</ul>

<style>
</style>

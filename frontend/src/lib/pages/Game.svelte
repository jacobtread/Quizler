<script lang="ts">
  import LobbyView from "$lib/components/LobbyView.svelte";
  import StartingView from "$lib/components/StartingView.svelte";
  import { setMessageHandler } from "$lib/socket";
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
    GameState,
    type Question,
    type Score,
    type ScoreMessage,
    type SessionId
  } from "$lib/socket/models";
  import { setHome, type GameData } from "$stores/state";

  export let gameData: GameData;

  // Hook the handlers for the different message types
  setMessageHandler(ServerMessage.OtherPlayer, onOtherPlayer);
  setMessageHandler(ServerMessage.GameState, onGameState);
  setMessageHandler(ServerMessage.TimeSync, onTimeSync);
  setMessageHandler(ServerMessage.Question, onQuestion);
  setMessageHandler(ServerMessage.Scores, onScores);
  setMessageHandler(ServerMessage.Score, onScore);
  setMessageHandler(ServerMessage.Error, onError);
  setMessageHandler(ServerMessage.Kicked, onKicked);

  let players: OtherPlayer[] = [];
  let gameState: GameState = GameState.Lobby;
  let question: Question | null = null;
  let score: Score | null = null;
  let scores: Record<SessionId, number> = {};

  let answered: boolean;

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
    question = msg.question;
  }

  function onScores(msg: ScoresMessage) {
    console.debug("Score message", msg);
    scores = msg.scores;
  }

  function onScore(msg: ScoreMessage) {
    console.debug("Score message", msg);
    score = msg.score;
  }

  function onError(msg: ErrorMessage) {
    console.error("Server error", msg.error);
  }

  function onKicked(msg: KickedMessage) {
    console.debug("Kick message", msg);
    // Remove from the players list
    players = players.filter((player) => player.id !== msg.session_id);

    // if the removed player was us
    if (msg.session_id === gameData.id) {
      // TODO: Display kicked message
      setHome();
    }
  }
</script>

{#if gameState === GameState.Lobby}
  <LobbyView {gameData} {players} />
{:else if gameState === GameState.Starting}
  <StartingView {gameData} />
{/if}

<style>
</style>

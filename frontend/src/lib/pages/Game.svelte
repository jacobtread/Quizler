<script lang="ts">
  import AnsweredView from "$lib/components/game/AnsweredView.svelte";
  import AwaitReadyView from "$lib/components/game/AwaitReadyView.svelte";
  import FinishedView from "$lib/components/game/FinishedView.svelte";
  import LobbyScoreView from "$lib/components/game/LobbyScoreView.svelte";
  import LobbyView from "$lib/components/game/LobbyView.svelte";
  import QuestionView from "$lib/components/game/QuestionView.svelte";
  import ScoreView from "$lib/components/game/ScoreView.svelte";
  import StartingView from "$lib/components/game/StartingView.svelte";
  import * as socket from "$lib/socket";
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
    type SessionId,
    type TimerState,
    ClientMessage,
    ScoreType
  } from "$lib/socket/models";
  import { formatImageUrl } from "$lib/utils";
  import { setHome, type GameData } from "$stores/state";

  export let gameData: GameData;

  // Hook the handlers for the different message types
  socket.setHandler(ServerMessage.OtherPlayer, onOtherPlayer);
  socket.setHandler(ServerMessage.GameState, onGameState);
  socket.setHandler(ServerMessage.TimeSync, onTimeSync);
  socket.setHandler(ServerMessage.Question, onQuestion);
  socket.setHandler(ServerMessage.Scores, onScores);
  socket.setHandler(ServerMessage.Score, onScore);
  socket.setHandler(ServerMessage.Error, onError);
  socket.setHandler(ServerMessage.Kicked, onKicked);

  let players: OtherPlayer[] = [];
  let gameState: GameState = GameState.Lobby;

  let question: Question | null = null;

  let score: Score = { ty: ScoreType.Incorrect };
  let scores: Record<SessionId, number> = {};

  let answered: boolean;

  let timer: TimerState = { total: 0, elapsed: 0 };
  let lastUpdateTime: number = 0;

  function updateTimer() {
    // Don't update the timer if we have reached the time
    if (timer.elapsed === timer.total) return;

    const time = performance.now();
    const elapsed = time - lastUpdateTime;

    timer.elapsed += elapsed;
    lastUpdateTime = time;

    if (timer.elapsed > timer.total) {
      timer.elapsed = timer.total;
    } else {
      // Request the next animation frame
      requestAnimationFrame(updateTimer);
    }
  }

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
    lastUpdateTime = performance.now();
    timer = { total: msg.total, elapsed: msg.elapsed };
    updateTimer();
  }

  function onQuestion(msg: QuestionMessage) {
    console.debug("Question message", msg);
    question = msg.question;
    score = { ty: ScoreType.Incorrect };

    if (msg.question.image !== null) {
      // Preload the image and then send the ready state
      const img = new Image();
      img.src = formatImageUrl(gameData.token, msg.question.image).toString();
      img.onload = () => {
        console.debug("Preloaded question image", img.src);
        onReady();
      };
    } else {
      onReady();
    }
  }

  async function onReady() {
    let res = await socket.send(ClientMessage.Ready, {});
    if (res.ty === ServerMessage.Error) {
      console.error("Error while attempting to cancel", res.error);
    }
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
  <StartingView {gameData} {timer} />
{:else if gameState === GameState.AwaitingReady}
  <AwaitReadyView />
{:else if gameState === GameState.AwaitingAnswers}
  {#if gameData.host}
    <LobbyScoreView {gameData} {players} {timer} {scores} />
  {:else if question !== null}
    {#if !answered}
      <QuestionView {question} {gameData} {timer} bind:answered />
    {:else}
      <AnsweredView />
    {/if}
  {/if}
{:else if gameState === GameState.Marked}
  {#if gameData.host}
    <LobbyScoreView {gameData} {players} {timer} {scores} />
  {:else}
    <ScoreView {gameData} {scores} {score} />
  {/if}
{:else if gameState === GameState.Finished}
  <FinishedView />
{/if}

<script context="module" lang="ts">
  import type { GameConfig, Scores } from "$api/models";

  export interface GameData {
    // ID of the current player
    id: SessionId;
    // Current game token
    token: string;
    // Current game config
    config: GameConfig;
    // Whether we are the host
    host: boolean;
    // The current player name
    name?: string | undefined;
  }
</script>

<script lang="ts">
  import {
    ServerEvent,
    type PlayerData,
    GameState,
    type Question,
    type Score,
    type SessionId,
    ScoreType,
    removeReasonText,
    type GameSummary,
    RemoveReason,
    ClientMessage
  } from "$api/models";

  import * as socket from "$api/socket";
  import { preloadImage } from "$api/http";

  import { errorDialog } from "$stores/dialogStore";
  import { setHome } from "$stores/state";

  import AnsweredView from "$pages/game/AnsweredView.svelte";
  import FinishedView from "$pages/game/FinishedView.svelte";
  import QuestionView from "$pages/game/QuestionView.svelte";
  import LobbyView from "$pages/game/LobbyView.svelte";
  import ScoreView from "$pages/game/ScoreView.svelte";
  import Waiting from "$pages/game/Waiting.svelte";
  import Starting from "$pages/game/Starting.svelte";
  import Loading from "$pages/Loading.svelte";

  export let gameData: GameData;

  let players: PlayerData[] = gameData.host
    ? []
    : [{ id: gameData.id, name: gameData.name ?? "" }];
  let gameState: GameState = GameState.Lobby;

  // The current game summary
  let summary: GameSummary | null = null;

  let question: Question | null = null;

  let score: Score = { ty: ScoreType.Incorrect };
  let scores: Scores = {};

  let answered: boolean;

  let timeMs: number = 0;
  let lastUpdateTime: number = 0;

  function updateTimer() {
    // Don't update the timer if we have reached the time
    if (timeMs <= 0) return;

    const time = performance.now();

    const elapsed = time - lastUpdateTime;

    timeMs -= elapsed;
    if (timeMs < 0) timeMs = 0;

    lastUpdateTime = time;

    if (timeMs != 0) {
      // Request the next animation frame
      requestAnimationFrame(updateTimer);
    }
  }

  // Hook the handlers for the different message types
  socket.setHandler(ServerEvent.PlayerData, (msg) => {
    console.debug("Other player message", msg);
    // Add to the players list
    players.push(msg);
    players = players;
  });

  socket.setHandler(ServerEvent.GameState, (msg) => {
    console.debug("Game state message", msg);
    gameState = msg.state;

    // If the state has changed reset our answered state
    answered = false;

    // Reset known scores when reverting to lobby state
    if (msg.state === GameState.Lobby) {
      scores = {};
    } else if (msg.state === GameState.Finished) {
      // Compute the finished summary
      const playersExt = players.map((player) => ({
        score: scores[player.id] ?? 0,
        ...player
      }));

      // Sort the players based on score
      playersExt.sort((a, b) => b.score - a.score);

      summary = {
        players: playersExt
      };
    }
  });

  socket.setHandler(ServerEvent.Timer, (msg) => {
    console.debug("Time sync message", msg);

    lastUpdateTime = performance.now();
    timeMs = msg.value;

    updateTimer();
  });

  socket.setHandler(ServerEvent.Question, async (msg) => {
    console.debug("Question message", msg);
    question = msg.question;

    // Preload the image
    const preloadedImage = await preloadImage(gameData.token, question);

    if (preloadedImage !== null && question.image !== null) {
      // Ensure browser compatability
      if (preloadedImage.decode !== undefined) {
        await preloadedImage.decode();
      }

      question.image.preloaded = preloadedImage;
    }

    // Update the ready state
    try {
      await socket.send({ ty: ClientMessage.Ready });
      console.debug("Server acknowledged ready state");
    } catch (e) {
      console.error("Error while attempting to ready", e);
    }
  });

  socket.setHandler(ServerEvent.Scores, (msg) => {
    console.debug("Score message", msg);
    scores = msg.scores;

    // Sort players list by the player scores
    let getScore = (id: number): number => scores[id] ?? 0;
    players = players.sort((a, b) => getScore(b.id) - getScore(a.id));
  });

  socket.setHandler(ServerEvent.Score, (msg) => {
    console.debug("Score message", msg);
    score = msg.score;
  });

  socket.setHandler(ServerEvent.Kicked, (msg) => {
    console.debug("Kick message", msg);
    // Remove from the players list
    players = players.filter((player) => player.id !== msg.id);

    // if the removed player was us
    if (msg.id === gameData.id) {
      setHome();

      // For remove reasons other than self disconnect
      if (msg.reason !== RemoveReason.Disconnected) {
        const reason = removeReasonText[msg.reason];
        errorDialog("Removed from game", reason);
      }
    }
  });
</script>

{#if gameState === GameState.Finished && summary != null}
  <FinishedView {gameData} {summary} />
{:else if gameState === GameState.Starting || gameState === GameState.PreQuestion || gameState === GameState.AwaitingReady}
  <Starting {gameState} {gameData} {timeMs} />
{:else if gameState === GameState.AwaitingAnswers && question != null}
  {#if !answered}
    <QuestionView {gameData} {question} {timeMs} bind:answered />
  {:else if players.length !== 1}
    <!-- 
      Don't bother showing answered screen if only one player 
      as it will just be a blink before the score screen 
    -->
    <AnsweredView />
  {/if}
{:else if gameData.host}
  <LobbyView {gameData} {gameState} {players} {scores} />
{:else if gameState === GameState.Lobby}
  <Waiting {gameData} />
{:else if gameState === GameState.Marked}
  <ScoreView {score} />
{:else}
  <!-- Just dot for the message while waiting for a state -->
  <Loading text="..." />
{/if}

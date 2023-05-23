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
    ServerMessage,
    type PlayerData,
    GameState,
    type Question,
    type Score,
    type SessionId,
    type TimerState,
    ScoreType,
    removeReasonText,
    type GameSummary,
    RemoveReason
  } from "$api/models";

  import * as socket from "$api/socket";
  import { setReady } from "$api/actions";
  import { preloadImage } from "$api/http";

  import { errorDialog } from "$stores/dialogStore";
  import { setHome } from "$stores/state";

  import Loading from "$pages/Loading.svelte";

  import AnsweredView from "$components/game/AnsweredView.svelte";
  import FinishedView from "$components/game/FinishedView.svelte";
  import QuestionView from "$components/game/QuestionView.svelte";
  import LobbyView from "$components/game/LobbyView.svelte";
  import ScoreView from "$components/game/ScoreView.svelte";

  import { onMount } from "svelte";

  export let gameData: GameData;

  // Reference to the preloaded image so the browser doesn't unload it
  // linting is disabled because this only stores the reference never using it
  // eslint-disable-next-line
  let preloadedImage: HTMLImageElement | null = null;

  let players: PlayerData[] = [];
  let gameState: GameState = GameState.Lobby;

  // The current game summary
  let summary: GameSummary | null = null;

  if (!gameData.host) {
    players.push({ id: gameData.id, name: gameData.name ?? "" });
  }

  let question: Question | null = null;

  let score: Score = { ty: ScoreType.Incorrect };
  let scores: Scores = {};

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

  // Hook the handlers for the different message types
  socket.setHandler(ServerMessage.PlayerData, (msg) => {
    console.debug("Other player message", msg);
    // Add to the players list
    players.push(msg);
    players = players;
  });

  socket.setHandler(ServerMessage.GameState, (msg) => {
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

  socket.setHandler(ServerMessage.TimeSync, (msg) => {
    console.debug("Time sync message", msg);
    lastUpdateTime = performance.now();
    timer = { total: msg.total, elapsed: msg.elapsed };
    updateTimer();
  });

  socket.setHandler(ServerMessage.Question, async (msg) => {
    console.debug("Question message", msg);
    question = msg.question;
    score = { ty: ScoreType.Incorrect };

    // Host doesn't need to load images
    if (gameData.host) return;

    // Preload the image
    preloadedImage = await preloadImage(gameData.token, question);

    if (preloadedImage !== null && question.image !== null) {
      // Prepare the image element for insertion
      preloadedImage.classList.add("question-image");
      preloadedImage.setAttribute("data-fit", question.image.fit);
      preloadedImage.alt = question.text;

      // Ensure browser compatability
      if (preloadedImage.decode !== undefined) {
        await preloadedImage.decode();
      }
    }

    // Update the ready state
    await setReady();

    console.debug("Server acknowledged ready state");
  });

  socket.setHandler(ServerMessage.Scores, (msg) => {
    console.debug("Score message", msg);
    scores = msg.scores;

    // Sort players list by the player scores
    let getScore = (id: number): number => scores[id] ?? 0;
    players = players.sort((a, b) => getScore(b.id) - getScore(a.id));
  });

  socket.setHandler(ServerMessage.Score, (msg) => {
    console.debug("Score message", msg);
    score = msg.score;
  });

  socket.setHandler(ServerMessage.Error, (msg) => {
    console.error("Server error", msg.error);
  });

  socket.setHandler(ServerMessage.Kicked, (msg) => {
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

  onMount(() => {
    // Attempt to make the browser fullscreen to make the app more visible
    // if (!gameData.host) tryFullscreen();
  });
</script>

{#if gameState === GameState.Finished && summary != null}
  <FinishedView {gameData} {summary} />
{:else if gameData.host || gameState === GameState.Lobby || gameState === GameState.Starting}
  <LobbyView {gameData} {gameState} {players} {timer} {scores} />
{:else if gameState === GameState.AwaitingReady}
  <Loading text="Waiting for other players..." />
{:else if gameState === GameState.AwaitingAnswers && question != null}
  {#if !answered}
    <QuestionView {question} {timer} {preloadedImage} bind:answered />
  {:else if players.length !== 1}
    <!-- 
      Don't bother showing answered screen if only one player 
      as it will just be a blink before the score screen 
    -->
    <AnsweredView />
  {/if}
{:else if gameState === GameState.Marked}
  {#if timer.elapsed >= timer.total * 0.25}
    <!--  If 1/4 of the wait timer has been elapsed show the lobby view -->
    <LobbyView {gameData} {gameState} {players} {timer} {scores} />
  {:else}
    <ScoreView {score} />
  {/if}
{:else}
  <!-- Just dot for the message while waiting for a state -->
  <Loading text="..." />
{/if}

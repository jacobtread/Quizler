<script lang="ts">
  import AnsweredView from "$lib/components/game/AnsweredView.svelte";
  import AwaitReadyView from "$lib/components/game/AwaitReadyView.svelte";
  import FinishedView from "$lib/components/game/FinishedView.svelte";
  import LobbyView from "$lib/components/game/LobbyView.svelte";
  import QuestionView from "$lib/components/game/QuestionView.svelte";
  import ScoreView from "$lib/components/game/ScoreView.svelte";
  import * as socket from "$lib/socket";
  import {
    ServerMessage,
    type PlayerData,
    GameState,
    type Question,
    type Score,
    type SessionId,
    type TimerState,
    ClientMessage,
    ScoreType,
    ServerError,
    removeReasonText
  } from "$lib/socket/models";
  import { errorDialog } from "$lib/stores/dialogStore";
  import { formatImageUrl } from "$lib/utils";
  import { setHome, type GameData } from "$stores/state";

  export let gameData: GameData;

  let players: PlayerData[] = [];
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

  async function setReady() {
    try {
      await socket.send({ ty: ClientMessage.Ready });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to ready", error);
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
    }
  });

  socket.setHandler(ServerMessage.TimeSync, (msg) => {
    console.debug("Time sync message", msg);
    lastUpdateTime = performance.now();
    timer = { total: msg.total, elapsed: msg.elapsed };
    updateTimer();
  });

  socket.setHandler(ServerMessage.Question, (msg) => {
    console.debug("Question message", msg);
    question = msg.question;
    score = { ty: ScoreType.Incorrect };

    if (msg.question.image !== null) {
      // Preload the image and then send the ready state
      const img = new Image();
      img.src = formatImageUrl(gameData.token, msg.question.image);
      img.onload = () => {
        console.debug("Preloaded question image", img.src);
        setReady();
      };
    } else {
      setReady();
    }
  });

  socket.setHandler(ServerMessage.Scores, (msg) => {
    console.debug("Score message", msg);
    scores = msg.scores;
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
      const reason = removeReasonText[msg.reason];
      errorDialog("Removed from game", reason);
    }
  });
</script>

{#if gameState === GameState.Finished}
  <FinishedView {gameData} />
{:else if gameData.host || gameState === GameState.Lobby || gameState === GameState.Starting}
  <LobbyView {gameData} {gameState} {players} {timer} {scores} />
{:else if gameState === GameState.AwaitingReady}
  <AwaitReadyView />
{:else if gameState === GameState.AwaitingAnswers && question != null}
  {#if !answered}
    <QuestionView {question} {gameData} {timer} bind:answered />
  {:else}
    <!-- Answer result screen -->
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
  <!-- TODO: Properly loading view for unknown states -->
  <h1>Loading...</h1>
{/if}

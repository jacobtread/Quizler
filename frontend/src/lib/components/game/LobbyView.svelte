<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import * as socket from "$lib/socket";
  import {
    ClientMessage,
    type PlayerData,
    HostAction,
    ServerError,
    type SessionId,
    type TimerState,
    GameState
  } from "$lib/socket/models";
  import { confirmDialog } from "$lib/stores/dialogStore";
  import { setHome, type GameData } from "$lib/stores/state";
  import { formatTime, getNumberWithOrdinal } from "$lib/utils";
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";
  import ScoreTweened from "../ScoreTweened.svelte";
  import Crown from "$lib/assets/icons/crown.svg";

  export let timer: TimerState;
  export let gameData: GameData;
  export let players: PlayerData[];
  export let scores: Record<SessionId, number>;
  export let gameState: GameState;

  async function doKick(id: number) {
    try {
      await socket.send({
        ty: ClientMessage.Kick,
        id
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to kick", error);
    }
  }

  async function doStart() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Start
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to start", error);
    }
  }

  async function doCancel() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Cancel
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to cancel", error);
    }
  }

  async function doSkip() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Skip
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to skip", error);
    }
  }

  async function doLeave() {
    if (gameData.host) {
      const result = await confirmDialog(
        "Confirm Leave",
        "Are you sure you want to leave? Leaving will remove all other players from the game"
      );

      if (!result) return;
    }

    // Kick self from game to leave
    await doKick(gameData.id);

    // Take back to the home scren
    setHome();
  }

  async function doReset() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Reset
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to reset", error);
    }
  }
</script>

<main class="main" transition:slide>
  <div class="quiz">
    <div class="head">
      <h1 class="token">
        {gameData.token}
      </h1>
      <div class="timing">
        {#if gameState === GameState.Starting}
          <p class="starting">Starting</p>
        {/if}

        {#if timer.elapsed !== timer.total}
          <p class="time">{formatTime(timer)}</p>
        {/if}
      </div>
    </div>

    <h2 class="name">{gameData.config.name}</h2>
    <p class="desc">{gameData.config.text}</p>

    <div class="actions">
      <button class="btn" on:click={doLeave}>Leave</button>

      {#if gameData.host}
        <!-- Theres an active timer add skip button -->
        {#if timer.elapsed !== timer.total}
          <button class="btn" on:click={doSkip}>Skip</button>
        {/if}

        {#if gameState === GameState.Starting}
          <!-- Cancel started button for starting games -->
          <button class="btn" on:click={doCancel}>Cancel</button>
        {:else if gameState === GameState.Finished}
          <!-- Restart started button for restarting games -->
          <button class="btn" on:click={doReset}>Restart</button>
        {:else if players.length > 0 && gameState === GameState.Lobby}
          <!-- Start button if theres players in the game -->
          <button class="btn" on:click={doStart}>Start</button>
        {/if}
      {/if}
    </div>

    <table class="players">
      <thead>
        <tr>
          <!-- Game over placing  -->
          {#if gameState === GameState.Finished}
            <th>Place</th>
          {/if}
          <th>Name</th>
          <th>Score</th>
          {#if gameData.host}
            <th>Actions</th>
          {/if}
        </tr>
      </thead>
      <tbody>
        {#each players as player, index (player.id)}
          <tr class="player" animate:flip>
            <!-- first 3 places winners -->
            {#if gameState === GameState.Finished}
              <td class="player__place">
                {#if index == 0}
                  <img src={Crown} alt="Winner Crown" class="crown" />
                {/if}
                {getNumberWithOrdinal(index + 1)}
              </td>
            {/if}

            <td class="player__name">
              {player.name}
            </td>
            <td class="player_score">
              <ScoreTweened value={scores[player.id] ?? 0} />
            </td>
            <!-- Host privilleges -->
            {#if gameData.host}
              <td class="player__action">
                <button class="btn" on:click={() => doKick(player.id)}>
                  Kick
                </button>
              </td>
            {/if}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</main>

<style lang="scss">
  @import "../../assets/scheme.scss";

  .head {
    display: flex;
    flex-flow: row nowrap;
  }

  .starting {
    font-size: 1rem;
    margin: 0 1rem;
    color: #999;
  }

  .timing {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .time {
    font-weight: bold;
    color: $primary;
    font-size: 3rem;
    text-align: center;
  }

  .quiz {
    width: 100%;
    max-width: 48rem;
  }

  .players {
    width: 100%;
    border-collapse: collapse;
    border: 1px solid $surfaceLight;
  }

  th {
    background-color: $surface;
    border: 1px solid $surfaceLight;
    padding: 1rem;
  }

  th:first-child {
    text-align: left;
  }

  td {
    background-color: $surface;
    border: 1px solid $surfaceLight;
    padding: 1rem;
  }

  .player {
    position: relative;
    background-color: $surface;
    border: 1px solid $surfaceLight;
  }

  .player__place {
    padding: 1rem;
    color: #fff;
    font-weight: bold;
  }

  .crown {
    position: absolute;
    width: 3rem;
    left: 0;
    top: 0;
    transform: translate(-30%, -50%) rotate(-30deg);
    animation: 0.5s 0.5s crown ease forwards;
    opacity: 0;
  }

  @keyframes crown {
    0% {
      opacity: 0;
      transform: translate(-30%, -50%) rotate(-30deg) scale(1);
    }
    30% {
      opacity: 1;
      transform: translate(-50%, -70%) rotate(-23deg) scale(1.5);
    }
    60% {
      opacity: 1;
      transform: translate(-40%, -60%) rotate(23deg) scale(1);
    }
    100% {
      opacity: 1;
      transform: translate(-30%, -50%) rotate(-30deg) scale(1);
    }
  }

  .player__name {
    width: 100%;
    padding: 1rem;
  }

  .token {
    color: $primary;
    font-weight: bold;
    font-size: 3rem;
    margin: 0;
    width: 100%;
  }

  .desc {
    margin-bottom: 1rem;
  }

  .player__action {
    padding: 0;
  }

  .actions {
    margin-bottom: 1rem;
    display: flex;
    gap: 1rem;
  }

  .actions .btn {
    flex: auto;
  }

  .main {
    display: flex;
    flex-flow: column;
    align-items: center;
    padding: 1rem;
  }
</style>

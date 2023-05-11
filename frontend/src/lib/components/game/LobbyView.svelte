<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import * as socket from "$lib/socket";
  import {
    ClientMessage,
    type OtherPlayer,
    HostAction,
    ServerError,
    type SessionId,
    type TimerState,
    GameState
  } from "$lib/socket/models";
  import { confirmDialog } from "$lib/stores/dialogStore";
  import { setHome, type GameData } from "$lib/stores/state";
  import { formatTime } from "$lib/utils";
  import { slide } from "svelte/transition";

  export let timer: TimerState;
  export let gameData: GameData;
  export let players: OtherPlayer[];
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
      <button class="button" on:click={doLeave}>Leave</button>

      {#if gameData.host}
        <!-- Theres an active timer add skip button -->
        {#if timer.elapsed !== timer.total}
          <button class="button" on:click={doSkip}>Skip</button>
        {/if}

        {#if gameState === GameState.Starting}
          <!-- Cancel started button for starting games -->
          <button class="button" on:click={doCancel}>Cancel</button>
        {:else if players.length > 0 && gameState === GameState.Lobby}
          <!-- Start button if theres players in the game -->
          <button class="button" on:click={doStart}>Start</button>
        {/if}
      {/if}
    </div>

    <table class="players">
      <thead>
        <tr>
          <th>Name</th>
          <th>Score</th>
          {#if gameData.host}
            <th>Actions</th>
          {/if}
        </tr>
      </thead>
      <tbody>
        {#each players as player}
          <tr class="player">
            <td class="player__name">{player.name}</td>
            <td class="player__name">{scores[player.id] ?? 0}</td>
            <!-- Host privilleges -->
            {#if gameData.host}
              <td class="player__action">
                <button class="button" on:click={() => doKick(player.id)}>
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
  .host {
    font-size: 1rem;
    background-color: $primary;
    padding: 0.5rem;
    vertical-align: middle;
    margin-left: 0.5rem;
    color: #ffffff;
  }

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

  .token__inner {
    text-align: right;
    margin-left: auto;
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
  }

  .player {
    background-color: $surface;
    border: 1px solid $surfaceLight;
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

  .actions {
    margin-bottom: 1rem;
    display: flex;
    gap: 1rem;
  }

  .actions .button {
    flex: auto;
  }

  .main {
    display: flex;
    flex-flow: column;
    align-items: center;
    padding: 1rem;
  }
</style>

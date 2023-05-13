<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import {
    type PlayerData,
    type SessionId,
    type TimerState,
    GameState
  } from "$lib/socket/models";
  import { confirmDialog } from "$lib/stores/dialogStore";
  import { setHome, type GameData } from "$lib/stores/state";
  import { formatTime } from "$lib/utils";
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";
  import ScoreTweened from "../ScoreTweened.svelte";
  import {
    doHostCancel,
    doHostSkip,
    doHostStart,
    doKick
  } from "$lib/socket/actions";

  export let timer: TimerState;
  export let gameData: GameData;
  export let players: PlayerData[];
  export let scores: Record<SessionId, number>;
  export let gameState: GameState;

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

    <div class="btn-row btn-row--fill">
      <button class="btn" on:click={doLeave}>Leave</button>

      {#if gameData.host}
        <!-- Theres an active timer add skip button -->
        {#if timer.elapsed !== timer.total}
          <button class="btn" on:click={doHostSkip}>Skip</button>
        {/if}

        {#if gameState === GameState.Starting}
          <!-- Cancel started button for starting games -->
          <button class="btn" on:click={doHostCancel}>Cancel</button>
        {:else if players.length > 0 && gameState === GameState.Lobby}
          <!-- Start button if theres players in the game -->
          <button class="btn" on:click={doHostStart}>Start</button>
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
        {#each players as player (player.id)}
          <tr class="player" animate:flip>
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

  .main {
    display: flex;
    flex-flow: column;
    align-items: center;
    padding: 1rem;
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

  .quiz {
    width: 100%;
    max-width: 48rem;
  }

  .players {
    margin-top: 1rem;
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
</style>

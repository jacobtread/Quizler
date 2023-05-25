<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";

  import { doHostAction, doKick, leave } from "$api/actions";
  import {
    type PlayerData,
    type SessionId,
    type TimerState,
    GameState,
    HostAction
  } from "$api/models";

  import { formatTime } from "$lib/utils/utils";

  import ScoreTweened from "$components/TweenedValue.svelte";
  import type { GameData } from "$pages/Game.svelte";

  export let timer: TimerState;
  export let gameData: GameData;
  export let players: PlayerData[];
  export let scores: Record<SessionId, number>;
  export let gameState: GameState;

  let skipCooldown = false;

  // Sends the host start action
  const start = () => doHostAction(HostAction.Start);

  // Sends the host cancel action
  const cancel = () => doHostAction(HostAction.Cancel);

  // Sends the host skip action
  const skip = () => {
    skipCooldown = true;
    doHostAction(HostAction.Skip);
    setTimeout(() => (skipCooldown = false), 300);
  };
</script>

<main class="page page--middle page--overflow" transition:slide>
  <div class="quiz">
    <div class="head">
      <h1 class="token">
        {gameData.token}
      </h1>
      <p class="timing">
        {#if gameState === GameState.Starting}
          <span class="starting">Starting</span>
        {/if}

        {#if timer.elapsed !== timer.total}
          <span class="time">{formatTime(timer)}</span>
        {/if}
      </p>
    </div>

    <h2 class="name">{gameData.config.name}</h2>
    <p class="desc">{gameData.config.text}</p>

    <div class="btn-row btn-row--fill actions">
      <button class="btn" on:click={() => leave(gameData)}>Leave</button>

      {#if gameData.host}
        <!-- Theres an active timer add skip button -->
        {#if gameState !== GameState.Lobby}
          <button
            class="btn"
            disabled={skipCooldown || timer.elapsed === timer.total}
            on:click={skip}>Skip</button
          >
        {/if}

        {#if gameState === GameState.Starting}
          <!-- Cancel started button for starting games -->
          <button class="btn" on:click={cancel}>Cancel</button>
        {:else if players.length > 0 && gameState === GameState.Lobby}
          <!-- Start button if theres players in the game -->
          <button class="btn" on:click={start}>Start</button>
        {/if}
      {/if}
    </div>

    <table>
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
  @import "../../../assets/scheme.scss";

  .page {
    padding: 1rem;
  }

  .head {
    display: flex;
    justify-content: space-between;
    width: 100%;
  }

  .timing {
    display: flex;
    flex: auto;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    text-align: right;
    font-size: 1rem;
    color: #999;
  }

  .time {
    float: right;
    font-weight: bold;
    color: $primary;
    font-size: 3rem;
    text-align: center;
    vertical-align: middle;
  }

  .quiz {
    width: 100%;
    max-width: 48rem;
  }

  .actions {
    margin-bottom: 1rem;
  }

  .player {
    position: relative;
    background-color: $surface;
    border: 1px solid $surfaceLight;

    &__name {
      width: 100%;
      padding: 1rem;
    }

    &__action {
      padding: 0;
    }
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
</style>

<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";

  import { doHostAction, doKick, leave } from "$api/actions";
  import {
    type PlayerData,
    type SessionId,
    GameState,
    HostAction
  } from "$api/models";

  import ScoreTweened from "$components/TweenedValue.svelte";
  import type { GameData } from "$pages/Game.svelte";

  export let gameData: GameData;
  export let players: PlayerData[];
  export let scores: Record<SessionId, number>;
  export let gameState: GameState;

  // Sends the next state action
  const next = () => doHostAction(HostAction.Next);
</script>

<main class="page page--middle page--overflow" transition:slide|global>
  <div class="quiz">
    <h1 class="token">
      {gameData.token}
    </h1>

    <h2 class="name">{gameData.config.name}</h2>
    <p class="desc">{gameData.config.text}</p>

    <div class="btn-row btn-row--fill actions">
      <button class="btn" on:click={() => leave(gameData)}>Leave</button>

      {#if gameState === GameState.Marked}
        <!-- Cancel started button for starting games -->
        <button class="btn" on:click={next}>Next</button>
      {:else if players.length > 0 && gameState === GameState.Lobby}
        <!-- Start button if theres players in the game -->
        <button class="btn" on:click={next}>Start</button>
      {/if}
    </div>

    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Score</th>
          <th>Actions</th>
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
            <td class="player__action">
              <button class="btn" on:click={() => doKick(player.id)}>
                Kick
              </button>
            </td>
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
      padding: 0.5rem;
      display: flex;
      flex-flow: row;
      justify-content: center;
      border: none;
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

<script lang="ts">
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";

  import { HostAction, type GameSummary } from "$api/models";
  import { doHostAction, leave } from "$api/actions";

  import ScoreTweened from "$components/TweenedValue.svelte";
  import Crown from "$components/icons/Crown.svelte";

  import type { GameData } from "$pages/Game.svelte";

  import { getNumberWithOrdinal } from "$lib/utils/utils";

  export let summary: GameSummary;
  export let gameData: GameData;

  const doHostReset = () => doHostAction(HostAction.Reset);
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

      {#if gameData.host}
        <!-- Restart started button for restarting games -->
        <button class="btn" on:click={doHostReset}>Restart</button>
      {/if}
    </div>

    <table>
      <thead>
        <tr>
          <th>Place</th>
          <th>Name</th>
          <th>Score</th>
        </tr>
      </thead>
      <tbody>
        {#each summary.players as player, index (player.id)}
          <tr class="player" animate:flip>
            <td class="player__place">
              {#if index == 0}
                <div class="crown"><Crown /></div>
              {/if}
              {getNumberWithOrdinal(index + 1)}
            </td>

            <td class="player__name">
              {player.name}
            </td>
            <td class="player_score">
              <ScoreTweened value={player.score} />
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

    &__place {
      padding: 1rem;
      color: $textPrimary;
      font-weight: bold;
    }

    &__name {
      width: 100%;
      padding: 1rem;
    }
  }

  .crown > :global(svg) {
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

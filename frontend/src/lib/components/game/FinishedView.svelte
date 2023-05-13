<script lang="ts">
  import type { GameSummary } from "$lib/socket/models";
  import { confirmDialog } from "$lib/stores/dialogStore";
  import { setHome, type GameData } from "$lib/stores/state";
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { getNumberWithOrdinal } from "$lib/utils";
  import ScoreTweened from "../ScoreTweened.svelte";
  import Crown from "$lib/assets/icons/crown.svg";
  import { doHostReset, doKick } from "$lib/socket/actions";

  export let summary: GameSummary;
  export let gameData: GameData;

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
    </div>

    <h2 class="name">{gameData.config.name}</h2>
    <p class="desc">{gameData.config.text}</p>

    <div class="btn-row btn-row--fill">
      <button class="btn" on:click={doLeave}>Leave</button>

      {#if gameData.host}
        <!-- Restart started button for restarting games -->
        <button class="btn" on:click={doHostReset}>Restart</button>
      {/if}
    </div>

    <table class="players">
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
                <img src={Crown} alt="Winner Crown" class="crown" />
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
</style>

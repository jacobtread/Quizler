<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";

  import { doHostAction, doKick } from "$api/actions";
  import {
    type PlayerData,
    type SessionId,
    type TimerState,
    GameState,
    HostAction
  } from "$api/models";

  import { confirmDialog } from "$stores/dialogStore";

  import { setHome } from "$stores/state";

  import { formatTime } from "$lib/utils/utils";

  import ScoreTweened from "$components/TweenedValue.svelte";
  import type { GameData } from "$pages/Game.svelte";

  export let timer: TimerState;
  export let gameData: GameData;
  export let players: PlayerData[];
  export let scores: Record<SessionId, number>;
  export let gameState: GameState;

  async function leave() {
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

  // Sends the host start action
  const start = () => doHostAction(HostAction.Start);

  // Sends the host cancel action
  const cancel = () => doHostAction(HostAction.Cancel);

  // Sends the host skip action
  const skip = () => doHostAction(HostAction.Skip);
</script>

<main class="main" transition:slide>
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

    <div class="btn-row btn-row--fill">
      <button class="btn" on:click={leave}>Leave</button>

      {#if gameData.host}
        <!-- Theres an active timer add skip button -->
        {#if timer.elapsed !== timer.total}
          <button class="btn" on:click={skip}>Skip</button>
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

  .main {
    position: relative;
    display: flex;
    flex-flow: column;
    align-items: center;
    padding: 1rem;
    overflow: auto;
    height: 100%;
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

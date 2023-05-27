<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { doHostAction } from "$api/actions";

  import type { GameData } from "$pages/Game.svelte";
  import { GameState, HostAction, type TimerState } from "$lib/api/models";
  import { formatTime } from "$lib/utils/utils";

  export let gameState: GameState;
  export let timer: TimerState;
  export let gameData: GameData;

  // Sends the next state action
  const next = () => doHostAction(HostAction.Next);

  // Sends the host cancel action
  const reset = () => doHostAction(HostAction.Reset);
</script>

<main class="page page--overflow" transition:slide>
  <div class="quiz">
    {#if timer.elapsed !== timer.total}
      <span class="time">{formatTime(timer)}</span>
    {/if}

    {#if gameState === GameState.Starting}
      <p class="text">Get ready, the quiz is about to start.</p>
    {:else}
      <p class="text">Get ready to answer</p>
    {/if}
  </div>

  <div class="bottom">
    <p class="token">{gameData.token}</p>

    {#if gameData.host}
      {#if gameState === GameState.Starting}
        <button class="btn btn--surface" on:click={reset}>Cancel</button>
      {/if}
      <button class="btn btn--surface" on:click={next}>Skip</button>
    {/if}
  </div>
</main>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .bottom {
    width: 100%;
    background-color: $surface;
    padding: 0.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 5px solid $surfaceLight;
    padding-left: 1rem;
    gap: 1rem;
  }

  .quiz {
    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;
    flex: auto;
    text-align: center;

    padding: 1rem;
    background: linear-gradient(to bottom right, $startingStart, $startingEnd);
  }

  .time {
    font-size: 3rem;

    color: #fff;
    text-shadow: 0 3px 1px darken($color: $startingStart, $amount: 25);
  }

  .time {
    font-size: 5rem;
  }

  .text {
    font-size: 1.25rem;

    margin-bottom: 1rem;
    color: lighten($color: $startingStart, $amount: 32);
  }

  .token {
    color: #fff;
    font-weight: bold;
    font-size: 1.5rem;
  }
</style>
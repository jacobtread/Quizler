<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { leave } from "$api/actions";

  import type { GameData } from "$pages/Game.svelte";
  import type { TimerState } from "$lib/api/models";
  import { formatTime } from "$lib/utils/utils";

  export let timer: TimerState;
  export let gameData: GameData;
</script>

<main class="page page--overflow" transition:slide>
  <div class="quiz">
    {#if timer.elapsed !== timer.total}
      <span class="time">{formatTime(timer)}</span>
    {/if}
    <p class="text">Get ready, the quiz is about to start.</p>
  </div>

  <div class="bottom">
    <p class="token">{gameData.token}</p>
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

  .title,
  .time {
    font-size: 3rem;

    color: #fff;
    text-shadow: 0 3px 1px darken($color: $startingStart, $amount: 25);
  }

  .time {
    font-size: 5rem;
  }

  .content {
    padding: 1rem;
    border-radius: 1rem;
    width: 100%;
    max-width: 32rem;
    margin: 1rem auto;
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

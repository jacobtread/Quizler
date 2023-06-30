<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { slide } from "svelte/transition";
  import { leave } from "$api/actions";

  import type { GameData } from "$pages/Game.svelte";

  export let gameData: GameData;
</script>

<main class="page page--overflow" transition:slide|global>
  <div class="quiz">
    <h1 class="title">Waiting for host...</h1>
    <p class="text">You're in, waiting for the host to start the quiz.</p>
  </div>

  <div class="bottom">
    <p class="token">{gameData.token}</p>

    <button class="btn btn--surface" on:click={() => leave(gameData)}>
      Leave
    </button>
  </div>
</main>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .quiz {
    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;
    flex: auto;
    text-align: center;

    padding: 1rem;
    background: linear-gradient(to bottom right, $partialStart, $partialEnd);
  }

  .title {
    font-size: 3rem;

    color: #fff;
    text-shadow: 0 3px 1px darken($color: $partialStart, $amount: 25);
  }

  .text {
    font-size: 1.25rem;

    margin-bottom: 1rem;
    color: lighten($color: $partialStart, $amount: 32);
  }

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

  .token {
    color: #fff;
    font-weight: bold;
    font-size: 1.5rem;
  }

  @media screen and (max-width: 48rem) {
    .title {
      font-size: 2rem;
    }
  }
</style>

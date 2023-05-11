<script lang="ts">
  import { ScoreType, type Score } from "$lib/socket/models";
  import { randomRange } from "$lib/utils";
  import correctMessages from "$lib/data/correctMessages.json";
  import incorrectMessages from "$lib/data/incorrectMessages.json";
  import partialMessages from "$lib/data/partialMessages.json";
  import { slide } from "svelte/transition";

  export let score: Score;

  const messages = {
    [ScoreType.Correct]: correctMessages,
    [ScoreType.Partial]: partialMessages,
    [ScoreType.Incorrect]: incorrectMessages
  }[score.ty];

  const message: string = messages[randomRange(0, messages.length) - 1];
</script>

<main class="main" data-type={score.ty} transition:slide>
  <p class="text">{message}</p>
  {#if score.ty === ScoreType.Correct}
    <p class="score">+{score.value}</p>
  {:else if score.ty === ScoreType.Partial}
    <p class="ratio">{score.count} / {score.total}</p>
    <p class="score">+{score.value}</p>
  {/if}
</main>

<style lang="scss">
  @import "../../assets/scheme";

  .text {
    color: #fff;
    text-shadow: 0 1px 2px #000;
    display: block;
    margin-bottom: 1rem;
  }

  .score {
    display: block;
    padding: 1rem;
    background-color: $surface;
    border-radius: 0.5rem;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.7);
    color: #fff;
  }

  .main {
    width: 100%;
    height: 100%;
    display: flex;
    flex-flow: column;
    gap: 1rem;
    justify-content: center;
    align-items: center;
    background: linear-gradient(to bottom right, $primary, $secondary);
  }

  .main[data-type="Correct"] {
    background: linear-gradient(to bottom right, $correctStart, $correctEnd);
  }

  .main[data-type="Partial"] {
    background: linear-gradient(to bottom right, $partialStart, $partialEnd);
  }

  .main[data-type="Incorrect"] {
    background: linear-gradient(
      to bottom right,
      $incorrectStart,
      $incorrectEnd
    );
  }
</style>

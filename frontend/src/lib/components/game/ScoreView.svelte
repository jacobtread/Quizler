<script lang="ts">
  import { slide } from "svelte/transition";
  import { tweened } from "svelte/motion";

  import { ScoreType, type Score } from "$lib/socket/models";

  import { randomRange } from "$lib/utils/utils";

  import correctMessages from "$lib/data/correctMessages.json";
  import incorrectMessages from "$lib/data/incorrectMessages.json";
  import partialMessages from "$lib/data/partialMessages.json";

  export let score: Score;

  const messages = {
    [ScoreType.Correct]: correctMessages,
    [ScoreType.Partial]: partialMessages,
    [ScoreType.Incorrect]: incorrectMessages
  }[score.ty];

  const message: string = messages[randomRange(0, messages.length - 1)];
  const value = tweened(0, {
    delay: 500
  });

  $: {
    if (score.ty === ScoreType.Correct || score.ty === ScoreType.Partial) {
      value.set(score.value);
    }
  }
</script>

<main class="main" data-type={score.ty} transition:slide>
  <h1 class="title">{score.ty}</h1>
  <p class="text">{message}</p>
  {#if score.ty === ScoreType.Correct}
    <p class="score">+{$value.toFixed(0)}</p>
  {:else if score.ty === ScoreType.Partial}
    <p class="ratio">{score.count} / {score.total}</p>
    <p class="score">+{$value.toFixed(0)}</p>
  {/if}
</main>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .text {
    color: #fff;
    text-shadow: 0 1px 2px #000;
    display: block;
    margin-bottom: 1rem;
  }

  .score {
    padding: 1rem;
    background-color: rgba(0, 0, 0, 0.3);
    border-radius: 0.5rem;
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

    .title,
    .text {
      text-shadow: 0 3px 1px darken($color: $primary, $amount: 15);
    }
  }

  .main[data-type="Correct"] {
    background: linear-gradient(to bottom right, $correctStart, $correctEnd);

    .title {
      text-shadow: 0 3px 1px darken($color: $correctEnd, $amount: 15);
    }

    .text {
      text-shadow: 0 2px 1px darken($color: $correctEnd, $amount: 15);
    }
  }

  .main[data-type="Partial"] {
    background: linear-gradient(to bottom right, $partialStart, $partialEnd);

    .title {
      text-shadow: 0 3px 1px darken($color: $partialStart, $amount: 15);
    }

    .text {
      text-shadow: 0 2px 1px darken($color: $partialStart, $amount: 15);
    }
  }

  .main[data-type="Incorrect"] {
    background: linear-gradient(
      to bottom right,
      $incorrectStart,
      $incorrectEnd
    );

    .title {
      text-shadow: 0 3px 1px darken($color: $incorrectEnd, $amount: 15);
    }
    .text {
      text-shadow: 0 2px 1px darken($color: $incorrectEnd, $amount: 15);
    }
  }

  .title {
    font-size: 3rem;
    color: #fff;
  }

  .text {
    font-size: 1.25rem;
    color: #fff;
  }
</style>

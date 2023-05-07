<script lang="ts">
  import { ScoreType, type Score, type SessionId } from "$lib/socket/models";
  import type { GameData } from "$lib/stores/state";

  export let gameData: GameData;
  export let scores: Record<SessionId, number>;
  export let score: Score;

  let ownScore: number = scores[gameData.id] ?? 0;
</script>

<p>Score: {ownScore}</p>

{#if score.ty === ScoreType.Correct}
  <p>Correct!</p>
  <p>+{score.value}</p>
{:else if score.ty === ScoreType.Partial}
  <p>Almost correct! ({score.count} / {score.total})</p>
  <p>+{score.value}</p>
{:else}
  <p>Incorrect</p>
{/if}

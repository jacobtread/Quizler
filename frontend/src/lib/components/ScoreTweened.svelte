<script lang="ts">
  import { tweened } from "svelte/motion";

  export let value: number;

  const valueTween = tweened(value);

  $: {
    valueTween.set(value);
  }
</script>

{$valueTween.toFixed(0)}

<script lang="ts">
  import { tweened } from "svelte/motion";

  export let value: number;

  const valueTween = tweened(0, {
    delay: 500
  });

  $: {
    valueTween.set(value);
  }
</script>

{$valueTween.toFixed(0)}

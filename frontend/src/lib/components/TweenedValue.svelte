<!-- Number component that produced a value that is tweened to its target -->
<script lang="ts">
  import { tweened, type Tweened } from "svelte/motion";

  // The actual value
  export let value: number;

  // Create a tweened version of the value
  const valueTween: Tweened<number> = tweened(0, { delay: 500 });

  // Update the tweened value when it changes
  $: valueTween.set(value);
</script>

{$valueTween.toFixed(0)}

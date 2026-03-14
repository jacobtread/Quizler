<script lang="ts">
  import Spinner from "$components/Spinner.svelte";

  interface Props {
    text: string;
  }

  const { text }: Props = $props();
</script>

<main class="page page--center page--overflow">
  <Spinner />
  <p class="text">{text}</p>
</main>

<style>
  .text {
    margin-top: 1rem;
  }
</style>

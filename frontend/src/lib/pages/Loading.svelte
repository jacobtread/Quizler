<script lang="ts">
  import Spinner from "$components/Spinner.svelte";

  export let text: string;
</script>

<main class="page page--center page--overflow">
  <Spinner />
  <p class="text">{text}</p>
</main>

<style lang="scss">
  .text {
    margin-top: 1rem;
  }
</style>

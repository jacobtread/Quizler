<script lang="ts">
  import Spinner from "$components/Spinner.svelte";

  export let text: string;
</script>

<main class="main">
  <Spinner />
  <p class="text">{text}</p>
</main>

<style lang="scss">
  .text {
    margin-top: 1rem;
  }

  .main {
    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;
    height: 100%;
  }
</style>

<script lang="ts">
  import Spinner from "$lib/components/Spinner.svelte";
</script>

<!-- Loading page displayed while the socket is connecting -->

<main class="main">
  <Spinner />
  <p class="text">Connecting to server...</p>
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

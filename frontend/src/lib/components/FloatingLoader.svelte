<script lang="ts">
  import Spinner from "$components/Spinner.svelte";

  import { fade } from "svelte/transition";

  export let text: string = "";
</script>

<main class="main" transition:fade>
  <Spinner />
  <p class="text">{text}</p>
</main>

<style lang="scss">
  .text {
    margin-top: 1rem;
  }

  .main {
    z-index: 2;
    position: fixed;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;

    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;

    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }
</style>

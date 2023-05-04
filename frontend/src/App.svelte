<script lang="ts">
  import Create from "./lib/Create.svelte";
  import ImageStorage from "./lib/ImageStorage.svelte";
  import { socketReady } from "./lib/socket";
</script>

<main class="main">
  {#if $socketReady}
    <Create />
    <ImageStorage />
  {:else}
    <p>Connecting to server...</p>
  {/if}
</main>

<style>
</style>

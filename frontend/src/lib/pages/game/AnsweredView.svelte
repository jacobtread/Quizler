<script lang="ts">
  import { getRandomMessage } from "$lib/utils/messages";
  import { slide } from "svelte/transition";

  const message = getRandomMessage("Answered");
</script>

<main class="main" transition:slide>
  <h1 class="title">Waiting...</h1>
  <p class="text">{message}</p>
</main>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .main {
    width: 100%;
    height: 100%;
    display: flex;
    flex-flow: column;
    justify-content: center;
    align-items: center;
    background: linear-gradient(to bottom right, $primary, $secondary);
  }

  .title {
    font-size: 3rem;

    color: #fff;
    text-shadow: 0 3px 1px darken($color: $primary, $amount: 15);
  }

  .text {
    font-size: 1.25rem;

    color: #fff;
    text-shadow: 0 3px 1px darken($color: $primary, $amount: 15);
  }
</style>

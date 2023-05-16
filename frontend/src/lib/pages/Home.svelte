<script lang="ts">
  import { setRoute } from "$components/Router.svelte";
  import { slide } from "svelte/transition";

  import GitHub from "$components/icons/GitHub.svelte";
  import Play from "$components/icons/Play.svelte";
  import Edit from "$components/icons/Edit.svelte";
  import Logo from "$components/icons/Logo.svelte";
</script>

<main class="main" transition:slide>
  <div class="left">
    <div class="logo">
      <Logo />
    </div>

    <a
      href="https://github.com/jacobtread/Quizler-v2"
      target="_blank"
      rel="noreferrer"
      title="View on Github"
      class="btn btn--icon github"
    >
      <GitHub />
      View on GitHub
    </a>
  </div>
  <div class="actions">
    <button
      on:click={() => setRoute("Connect")}
      class="action"
      aria-label="Join"
    >
      <Play />
      <div class="action__body">
        <p class="action__name">Join a quiz</p>
        <p class="action__text">Enter a game code and hop right in</p>
      </div>
    </button>

    <button
      on:click={() => setRoute("Create")}
      class="action"
      aria-label="Create"
    >
      <Edit />
      <div class="action__body">
        <p class="action__name">Create a quiz</p>
        <p class="action__text">Create your own quiz</p>
      </div>
    </button>
  </div>
</main>

<style lang="scss">
  @use "sass:color";
  @import "../../assets/scheme.scss";

  .left {
    text-align: center;
  }

  .logo {
    margin: 1rem auto;
  }

  .github {
    display: inline-block;
    padding: 0.75rem;
    border-radius: 0.5rem;
  }

  .main {
    display: flex;
    justify-content: center;
    align-items: center;
    overflow: auto;

    gap: 3rem;
    height: 100%;
  }

  .logo :global(> svg) {
    max-width: 16rem;
    padding: 1rem;
  }

  .actions {
    display: grid;
    gap: 1rem;
  }

  .action {
    position: relative;
    overflow: hidden;

    display: grid;
    grid-template-columns: min-content auto;

    align-items: center;
    gap: 1rem;

    padding: 1rem;
    padding-right: 2rem;

    border-radius: 1rem;

    background-color: $surface;
    border: 5px solid $surface;
    text-align: left;

    cursor: pointer;

    transition: background-color 0.5s ease, color 0.2s linear;

    :global(> svg) {
      padding: 1rem;
      box-sizing: content-box;
      background-color: $surfaceLight;
      border-radius: 0.5rem;
      transition: background-color 0.5s ease, color 0.2s linear;
    }

    &:before {
      content: "";

      position: absolute;
      left: 0;
      top: 0;
      width: 100%;
      height: 100%;
      z-index: -1;

      background: $primary;
      border-radius: 0.5rem;

      transform: translate(-100%);

      transition: 0.5s ease;
    }

    &:hover {
      // Adjust background alpha for background animation
      background-color: color.change($surface, $alpha: 0.62);

      &:before {
        transform: translate(0);
      }

      :global(> svg) {
        background-color: $primary;
      }
    }
  }

  .action__name {
    font-size: 1.4rem;
    margin-bottom: 0.25rem;
    color: $textPrimary;
  }

  .action__text {
    color: $textSecondary;
    font-size: 1.1rem;
  }

  @media screen and (max-height: 48rem) and (max-width: 52rem) {
    .main {
      justify-content: start;
      align-items: center;
      padding: 1rem 0;
      gap: 1rem;
    }
  }

  @media screen and (max-width: 52rem) {
    .main {
      flex-flow: column;
      gap: 1rem;
    }
  }

  @media screen and (max-width: 32rem) {
    .action {
      grid-template-columns: 1fr;
      padding-right: 1rem;

      :global(> svg) {
        display: block;
        width: calc(100% - 2rem);
        max-height: 3rem;
      }
    }

    .action__body {
      display: block;
      width: 100%;
    }
  }

  @media screen and (max-width: 16rem) {
    .action {
      padding-right: 1rem;

      :global(> svg) {
        padding: 1rem;
        width: auto;
      }
    }

    .action__body {
      display: none;
    }
  }
</style>

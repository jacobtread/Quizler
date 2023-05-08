<script lang="ts">
  import { setConnect, setCreate } from "$lib/stores/state";
  import Logo from "$lib/assets/logo.svg";
  import Play from "$lib/assets/icons/play.svg";
  import Edit from "$lib/assets/icons/edit.svg";
  import GitHub from "$lib/assets/icons/github.svg";
  import { slide } from "svelte/transition";
</script>

<main class="main" transition:slide>
  <img src={Logo} alt="Quizler Logo" class="logo" width="248" height="135" />
  <div class="actions">
    <button on:click={setConnect} class="action" aria-label="Join">
      <img src={Play} alt="Join Icon" class="action__icon" />
      <div>
        <span class="action__name">Join a quiz</span>
        <p class="action__text">Enter a game code and hop right in</p>
      </div>
    </button>

    <button on:click={setCreate} class="action" aria-label="Create">
      <img src={Edit} alt="Create Icon" class="action__icon" />
      <div>
        <span class="action__name">Create a quiz</span>
        <p class="action__text">Create your own quiz</p>
      </div>
    </button>
  </div>

  <a
    href="https://github.com/jacobtread/Quizler-v2"
    target="_blank"
    rel="noreferrer"
    title="View on Github"
    class="github"
  >
    <img src={GitHub} alt="GitHub Icon" class="github__icon" />
  </a>
</main>

<style lang="scss">
  @use "sass:color";
  @import "../assets/scheme.scss";

  .github {
    position: absolute;
    right: 1rem;
    bottom: 1rem;

    display: flex;
    gap: 1rem;
    text-decoration: none;
    background: $surface;
    padding: 1rem;
    border-radius: 2rem;
  }

  .main {
    display: flex;
    justify-content: center;
    align-items: center;

    gap: 3rem;
    height: 100%;
  }

  .logo {
    max-width: 16rem;
    padding: 1rem;
  }

  .actions {
    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  .action {
    position: relative;
    overflow: hidden;

    display: flex;
    flex-flow: row;
    align-items: center;
    gap: 1rem;

    width: 100%;

    padding: 1rem;
    padding-right: 2rem;

    border-radius: 2rem;

    background-color: $surface;
    border: 5px solid $surface;

    text-align: left;

    cursor: pointer;

    transition: background-color 0.5s ease, color 0.2s linear;

    &:before {
      content: "";

      position: absolute;
      left: 0;
      top: 0;
      width: 100%;
      height: 100%;
      z-index: -1;

      background: $primary;
      border-radius: 1.5rem;

      transform: translate(-100%);

      transition: 0.5s ease;
    }

    &:hover {
      // Adjust background alpha for background animation
      background-color: color.change($surface, $alpha: 0.62);

      &:before {
        transform: translate(0);
      }
    }
  }

  .action__icon {
    padding: 1rem;
    box-sizing: content-box;
    background-color: $surfaceLight;
    border-radius: 1.5rem;
    transition: background-color 0.5s ease, color 0.2s linear;
  }

  .action:hover .action__icon {
    background-color: $primary;
  }

  .action__name {
    font-size: 1.4rem;
    margin-bottom: 0.25rem;
    display: block;
    color: $textPrimary;
  }

  .action__text {
    color: $textSecondary;
    font-size: 1.1rem;
  }
</style>

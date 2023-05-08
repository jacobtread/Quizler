<script lang="ts">
  import { setConnect, setCreate } from "$lib/stores/state";
  import Logo from "$lib/assets/logo.svg";
  import Play from "$lib/assets/icons/play.svg";
  import Edit from "$lib/assets/icons/edit.svg";
  import { slide } from "svelte/transition";
</script>

<main class="main" transition:slide>
  <img src={Logo} alt="Quizler Logo" class="logo" />
  <div class="paths">
    <button on:click={setConnect} class="path" aria-label="Join">
      <img src={Play} alt="Join Icon" class="path__icon" />
      <div>
        <span class="path__name">Join a quiz</span>
        <p class="path__text">Enter a game code and hop right in</p>
      </div>
    </button>

    <button on:click={setCreate} class="path" aria-label="Create">
      <img src={Edit} alt="Create Icon" class="path__icon" />
      <div>
        <span class="path__name">Create a quiz</span>
        <p class="path__text">Create your own quiz</p>
      </div>
    </button>
  </div>
</main>

<style lang="scss">
  @import "../assets/scheme.scss";

  :global(html, body, #app) {
    height: 100%;
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

  .paths {
    display: flex;
    flex-flow: column;
    gap: 1rem;
    flex: auto;
    max-width: 31rem;
  }

  .path {
    position: relative;
    overflow: hidden;

    display: flex;
    flex-flow: row;
    gap: 1rem;
    color: #fff;
    background-color: #222;
    width: 100%;
    align-items: center;
    background-color: #222;
    color: #fff;
    padding: 1rem;
    border-radius: 2rem;
    border: 5px solid #222;
    text-align: left;
    cursor: pointer;

    transition: background-color 0.5s ease, color 0.2s linear;
  }

  .path:hover {
    background: rgba(34, 34, 34, 0.62);
  }

  .path:hover:before {
    transform: translate(0);
  }

  .path:before {
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    content: "";
    background: #f66828;
    border-radius: 1.5rem;
    transform: translate(-100%);
    transition: 0.5s ease;
    z-index: -1;
  }

  .path__icon {
    padding: 1rem;
    box-sizing: content-box;
    background-color: #333;
    border-radius: 1.5rem;
    transition: background-color 0.5s ease, color 0.2s linear;
  }

  .path:hover .path__icon {
    background-color: #f66828;
  }

  .path__name {
    font-size: 1.4rem;
    margin-bottom: 0.5rem;
    display: block;
  }

  .path__text {
    color: #ccc;
    font-size: 1.1rem;
  }
</style>

<script lang="ts">
  import { setConnect, setCreate } from "$stores/state";
  import { slide } from "svelte/transition";

  import GitHub from "$components/icons/GitHub.svelte";
  import Play from "$components/icons/Play.svelte";
  import Edit from "$components/icons/Edit.svelte";
  import Logo from "$components/icons/Logo.svelte";
</script>

<main class="main" transition:slide|global>
  <div class="left">
    <div class="logo">
      <Logo />
    </div>

    <a
      href="https://github.com/jacobtread/Quizler"
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
    <button on:click={setConnect} class="action" aria-label="Join">
      <Play />
      <div class="action__body">
        <p class="action__name">Join a quiz</p>
        <p class="action__text">Enter a game code and hop right in</p>
      </div>
    </button>

    <button on:click={setCreate} class="action" aria-label="Create">
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
    background-color: $appBackground;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='192' height='192' viewBox='0 0 192 192'%3E%3Cpath fill='%23444444' fill-opacity='0.4' d='M192 15v2a11 11 0 0 0-11 11c0 1.94 1.16 4.75 2.53 6.11l2.36 2.36a6.93 6.93 0 0 1 1.22 7.56l-.43.84a8.08 8.08 0 0 1-6.66 4.13H145v35.02a6.1 6.1 0 0 0 3.03 4.87l.84.43c1.58.79 4 .4 5.24-.85l2.36-2.36a12.04 12.04 0 0 1 7.51-3.11 13 13 0 1 1 .02 26 12 12 0 0 1-7.53-3.11l-2.36-2.36a4.93 4.93 0 0 0-5.24-.85l-.84.43a6.1 6.1 0 0 0-3.03 4.87V143h35.02a8.08 8.08 0 0 1 6.66 4.13l.43.84a6.91 6.91 0 0 1-1.22 7.56l-2.36 2.36A10.06 10.06 0 0 0 181 164a11 11 0 0 0 11 11v2a13 13 0 0 1-13-13 12 12 0 0 1 3.11-7.53l2.36-2.36a4.93 4.93 0 0 0 .85-5.24l-.43-.84a6.1 6.1 0 0 0-4.87-3.03H145v35.02a8.08 8.08 0 0 1-4.13 6.66l-.84.43a6.91 6.91 0 0 1-7.56-1.22l-2.36-2.36A10.06 10.06 0 0 0 124 181a11 11 0 0 0-11 11h-2a13 13 0 0 1 13-13c2.47 0 5.79 1.37 7.53 3.11l2.36 2.36a4.94 4.94 0 0 0 5.24.85l.84-.43a6.1 6.1 0 0 0 3.03-4.87V145h-35.02a8.08 8.08 0 0 1-6.66-4.13l-.43-.84a6.91 6.91 0 0 1 1.22-7.56l2.36-2.36A10.06 10.06 0 0 0 107 124a11 11 0 0 0-22 0c0 1.94 1.16 4.75 2.53 6.11l2.36 2.36a6.93 6.93 0 0 1 1.22 7.56l-.43.84a8.08 8.08 0 0 1-6.66 4.13H49v35.02a6.1 6.1 0 0 0 3.03 4.87l.84.43c1.58.79 4 .4 5.24-.85l2.36-2.36a12.04 12.04 0 0 1 7.51-3.11A13 13 0 0 1 81 192h-2a11 11 0 0 0-11-11c-1.94 0-4.75 1.16-6.11 2.53l-2.36 2.36a6.93 6.93 0 0 1-7.56 1.22l-.84-.43a8.08 8.08 0 0 1-4.13-6.66V145H11.98a6.1 6.1 0 0 0-4.87 3.03l-.43.84c-.79 1.58-.4 4 .85 5.24l2.36 2.36a12.04 12.04 0 0 1 3.11 7.51A13 13 0 0 1 0 177v-2a11 11 0 0 0 11-11c0-1.94-1.16-4.75-2.53-6.11l-2.36-2.36a6.93 6.93 0 0 1-1.22-7.56l.43-.84a8.08 8.08 0 0 1 6.66-4.13H47v-35.02a6.1 6.1 0 0 0-3.03-4.87l-.84-.43c-1.59-.8-4-.4-5.24.85l-2.36 2.36A12 12 0 0 1 28 109a13 13 0 1 1 0-26c2.47 0 5.79 1.37 7.53 3.11l2.36 2.36a4.94 4.94 0 0 0 5.24.85l.84-.43A6.1 6.1 0 0 0 47 84.02V49H11.98a8.08 8.08 0 0 1-6.66-4.13l-.43-.84a6.91 6.91 0 0 1 1.22-7.56l2.36-2.36A10.06 10.06 0 0 0 11 28 11 11 0 0 0 0 17v-2a13 13 0 0 1 13 13c0 2.47-1.37 5.79-3.11 7.53l-2.36 2.36a4.94 4.94 0 0 0-.85 5.24l.43.84A6.1 6.1 0 0 0 11.98 47H47V11.98a8.08 8.08 0 0 1 4.13-6.66l.84-.43a6.91 6.91 0 0 1 7.56 1.22l2.36 2.36A10.06 10.06 0 0 0 68 11 11 11 0 0 0 79 0h2a13 13 0 0 1-13 13 12 12 0 0 1-7.53-3.11l-2.36-2.36a4.93 4.93 0 0 0-5.24-.85l-.84.43A6.1 6.1 0 0 0 49 11.98V47h35.02a8.08 8.08 0 0 1 6.66 4.13l.43.84a6.91 6.91 0 0 1-1.22 7.56l-2.36 2.36A10.06 10.06 0 0 0 85 68a11 11 0 0 0 22 0c0-1.94-1.16-4.75-2.53-6.11l-2.36-2.36a6.93 6.93 0 0 1-1.22-7.56l.43-.84a8.08 8.08 0 0 1 6.66-4.13H143V11.98a6.1 6.1 0 0 0-3.03-4.87l-.84-.43c-1.59-.8-4-.4-5.24.85l-2.36 2.36A12 12 0 0 1 124 13a13 13 0 0 1-13-13h2a11 11 0 0 0 11 11c1.94 0 4.75-1.16 6.11-2.53l2.36-2.36a6.93 6.93 0 0 1 7.56-1.22l.84.43a8.08 8.08 0 0 1 4.13 6.66V47h35.02a6.1 6.1 0 0 0 4.87-3.03l.43-.84c.8-1.59.4-4-.85-5.24l-2.36-2.36A12 12 0 0 1 179 28a13 13 0 0 1 13-13zM84.02 143a6.1 6.1 0 0 0 4.87-3.03l.43-.84c.8-1.59.4-4-.85-5.24l-2.36-2.36A12 12 0 0 1 83 124a13 13 0 1 1 26 0c0 2.47-1.37 5.79-3.11 7.53l-2.36 2.36a4.94 4.94 0 0 0-.85 5.24l.43.84a6.1 6.1 0 0 0 4.87 3.03H143v-35.02a8.08 8.08 0 0 1 4.13-6.66l.84-.43a6.91 6.91 0 0 1 7.56 1.22l2.36 2.36A10.06 10.06 0 0 0 164 107a11 11 0 0 0 0-22c-1.94 0-4.75 1.16-6.11 2.53l-2.36 2.36a6.93 6.93 0 0 1-7.56 1.22l-.84-.43a8.08 8.08 0 0 1-4.13-6.66V49h-35.02a6.1 6.1 0 0 0-4.87 3.03l-.43.84c-.79 1.58-.4 4 .85 5.24l2.36 2.36a12.04 12.04 0 0 1 3.11 7.51A13 13 0 1 1 83 68a12 12 0 0 1 3.11-7.53l2.36-2.36a4.93 4.93 0 0 0 .85-5.24l-.43-.84A6.1 6.1 0 0 0 84.02 49H49v35.02a8.08 8.08 0 0 1-4.13 6.66l-.84.43a6.91 6.91 0 0 1-7.56-1.22l-2.36-2.36A10.06 10.06 0 0 0 28 85a11 11 0 0 0 0 22c1.94 0 4.75-1.16 6.11-2.53l2.36-2.36a6.93 6.93 0 0 1 7.56-1.22l.84.43a8.08 8.08 0 0 1 4.13 6.66V143h35.02z'%3E%3C/path%3E%3C/svg%3E");

    gap: 3rem;
    height: 100%;
  }

  .logo :global(> svg) {
    max-width: 16rem;
    padding: 1rem;
    fill: $logoFill;
  }

  .action {
    position: relative;
    overflow: hidden;

    width: 100%;

    align-items: center;
    gap: 1rem;

    padding: 1rem;
    margin-bottom: 1rem;

    border-radius: 1rem;

    background-color: $surface;
    border: 0.15rem solid $btnBorderColor;
    text-align: left;

    z-index: 2;

    cursor: pointer;

    transition: background-color 0.5s ease, color 0.2s linear;

    :global(> svg) {
      float: left;
      margin-right: 1rem;
      padding: 1rem;
      box-sizing: content-box;
      background-color: $surfaceLight;
      border-radius: 0.5rem;
      transition: background-color 0.5s ease, color 0.2s linear;
    }

    &__name {
      font-size: 1.4rem;
      margin-bottom: 0.25rem;
      color: $textPrimary;
    }

    &__text {
      color: $textSecondary;
      font-size: 1.1rem;
    }

    &:before {
      content: "";

      position: absolute;
      left: 0;
      top: 0;
      width: 100%;
      height: 100%;
      z-index: -1;

      background: $primaryDark;
      border-radius: 1rem;

      transform: translate(-100%);

      transition: 0.5s ease;
    }

    &:hover {
      &:before {
        transform: translate(0);
      }

      border-color: $primary;

      :global(> svg) {
        background-color: $primary;
      }
    }
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
    .actions {
      margin: 1rem;
    }

    .action :global(> svg) {
      display: block;
      float: none;
      width: calc(100% - 2rem);
      max-height: 3rem;
      margin-bottom: 1rem;
    }

    .action__body {
      display: block;
      width: 100%;
    }
  }

  @media screen and (max-width: 16rem) {
    .logo {
      width: 100%;
      overflow: hidden;
      padding: 0 1rem;
      margin: 0 auto;

      :global(> svg) {
        width: 100%;
      }
    }

    .actions {
      text-align: center;
      margin: 0;
    }

    .action {
      width: auto;

      :global(> svg) {
        padding: 1rem;
        width: auto;
        margin-bottom: 0;
        margin-right: 0;
      }

      &__body {
        display: none;
      }
    }

    .github {
      display: none;
    }
  }

  @media screen and (max-height: 48rem) {
    .github {
      display: none;
    }
  }

  @media screen and (max-height: 42rem) {
    .logo :global(> svg) {
      max-height: 6rem;
      max-width: none;
      padding: 1rem;
    }

    .logo {
      margin: 0 auto;
    }

    .main {
      gap: 0;
    }
  }

  @media screen and (max-height: 35rem) {
    .logo {
      display: none;
    }
  }
</style>

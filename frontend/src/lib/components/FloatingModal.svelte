<script context="module" lang="ts">
  export const enum ModelSize {
    Normal,
    Small
  }
</script>

<script lang="ts">
  import { fade, slide } from "svelte/transition";
  import Close from "$components/icons/Delete.svelte";

  export let visible: boolean;
  export let size: ModelSize = ModelSize.Normal;
</script>

{#if visible}
  <div class="floating-wrapper" transition:fade={{ duration: 200 }}>
    <div
      class="dialog"
      class:dialog--small={size == ModelSize.Small}
      transition:slide={{ duration: 200 }}
    >
      <button
        on:click={() => (visible = false)}
        class="btn btn--icon btn--surface"
      >
        <Close />
        Close
      </button>

      <slot />
    </div>
  </div>
{/if}

<style lang="scss">
  @import "../../assets/scheme.scss";

  .floating-wrapper {
    z-index: 1;
    position: fixed;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(5px);
    -webkit-backdrop-filter: blur(5px);
  }

  .dialog {
    background-color: $surface;

    border-radius: 0.5rem;

    width: 100%;
    max-width: 48rem;

    margin: 1rem;
    padding: 1rem;

    display: flex;
    flex-flow: column;
    gap: 1rem;

    &--small {
      max-width: 32rem;
    }
  }

  @media screen and (max-width: 48rem), (max-height: 48em) {
    .floating-wrapper {
      align-items: flex-start;
    }
  }
</style>

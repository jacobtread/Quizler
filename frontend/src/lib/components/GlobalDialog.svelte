<!-- Global dialog that can be called -->

<script lang="ts">
  import { fade, slide } from "svelte/transition";
  import { DialogType, consumeDialog, dialogStore } from "$stores/dialogStore";
</script>

{#if $dialogStore}
  <div class="floating-wrapper" transition:fade={{ duration: 200 }}>
    <div class="dialog" transition:slide|global={{ duration: 200 }}>
      <h1 class="dialog__title">{$dialogStore.title}</h1>

      <div class="dialog__msgs">
        {#each $dialogStore.message.split("\n") as line}
          <p class="dialog__msg">{line}</p>
        {/each}
      </div>

      <div class="btn-row btn-row--fill">
        {#if $dialogStore.ty === DialogType.Error}
          <button class="btn" on:click={() => consumeDialog(true)}>
            Close
          </button>
        {:else}
          <button class="btn" on:click={() => consumeDialog(true)}>
            Confirm
          </button>
          <button class="btn" on:click={() => consumeDialog(false)}>
            Cancel
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  @import "../../assets/scheme.scss";

  .dialog {
    background-color: $appBackground;
    border: 1px solid $surface;

    padding: 1rem;
    border-radius: 0.5rem;
    max-width: 32rem;
    width: 100%;
    margin: 1rem;

    &__title {
      margin-bottom: 0.5rem;
      color: #ffffff;
    }

    &__msg {
      margin-bottom: 0.25rem;
    }

    &__msgs {
      display: flex;
      flex-flow: column;
      // gap: 0.25rem?;
      max-height: 24rem;
      overflow: auto;
      margin-bottom: 1rem;
    }
  }
</style>

<!-- Global dialog that can be called -->

<script lang="ts">
  import {
    DialogType,
    consumeDialog,
    dialogStore
  } from "$lib/stores/dialogStore";
  import { fade, slide } from "svelte/transition";
</script>

{#if $dialogStore}
  <div class="dialog-wrapper" transition:fade={{ duration: 200 }}>
    <div class="dialog" transition:slide={{ duration: 200 }}>
      <h1>{$dialogStore.title}</h1>
      <p>{$dialogStore.message}</p>

      {#if $dialogStore.ty === DialogType.Error}
        <button on:click={() => consumeDialog(true)}>Close</button>
      {:else}
        <button on:click={() => consumeDialog(true)}>Confirm</button>
        <button on:click={() => consumeDialog(false)}>Cancel</button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .dialog-wrapper {
    position: fixed;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.7);
  }

  .dialog {
    background-color: #111;
    padding: 1rem;
  }
</style>

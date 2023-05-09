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
      <h1 class="dialog__title">{$dialogStore.title}</h1>
      <p class="dialog__msg">{$dialogStore.message}</p>

      <div class="dialog__actions">
        {#if $dialogStore.ty === DialogType.Error}
          <button class="button" on:click={() => consumeDialog(true)}
            >Close</button
          >
        {:else}
          <button class="button" on:click={() => consumeDialog(true)}
            >Confirm</button
          >
          <button class="button" on:click={() => consumeDialog(false)}
            >Cancel</button
          >
        {/if}
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  @import "../assets/scheme";

  .dialog-wrapper {
    z-index: 2;
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
    padding: 1rem;
    border-radius: 0.5rem;
    max-width: 32rem;
    width: 100%;
  }

  .button {
    background-color: $surfaceLight;
    flex: auto;
  }

  .dialog__title {
    margin-bottom: 0.5rem;
    color: #ffffff;
  }

  .dialog__msg {
    margin-bottom: 1rem;
  }

  .dialog__actions {
    display: flex;
    gap: 1rem;
  }
</style>

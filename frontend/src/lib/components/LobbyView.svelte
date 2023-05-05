<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import { sendMessage } from "$lib/socket";
  import { ClientMessageType, type OtherPlayer } from "$lib/socket/models";
  import type { GameData } from "$lib/stores/state";

  export let gameData: GameData;
  export let players: OtherPlayer[];

  async function doKick(player: OtherPlayer) {
    sendMessage({
      ty: ClientMessageType.Kick,
      id: player.id
    });
  }
</script>

{#if gameData.host}
  <p class="token">
    {gameData.token}
  </p>
{/if}
<h1>
  {gameData.config.basic.name}
  {#if gameData.host}
    <span class="host">Host</span>
  {/if}
</h1>
<p>{gameData.config.basic.text}</p>

<ul>
  {#each players as player}
    <li>
      <span>{player.name}</span>

      <!-- Host privilleges -->
      {#if gameData.host}
        <button on:click={() => doKick(player)}>Kick</button>
      {/if}
    </li>
  {/each}
</ul>

<style>
  .host {
    font-size: 1rem;
    background-color: #ff5500;
    padding: 0.5rem;
    vertical-align: middle;
    margin-left: 0.5rem;
  }

  .token {
    color: #ff5500;
    font-weight: bold;
    font-size: 3rem;
    margin: 0;
  }
</style>

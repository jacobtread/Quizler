<!-- Game view when the state is in the "Lobby" -->
<script lang="ts">
  import * as socket from "$lib/socket";
  import {
    ClientMessage,
    type OtherPlayer,
    HostAction,
    ServerError
  } from "$lib/socket/models";
  import type { GameData } from "$lib/stores/state";

  export let gameData: GameData;
  export let players: OtherPlayer[];

  async function doKick(player: OtherPlayer) {
    try {
      await socket.send({
        ty: ClientMessage.Kick,
        id: player.id
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to kick", error);
    }
  }

  async function doStart() {
    try {
      await socket.send({
        ty: ClientMessage.HostAction,
        action: HostAction.Start
      });
    } catch (e) {
      const error = e as ServerError;
      console.error("Error while attempting to start", error);
    }
  }
</script>

{#if gameData.host}
  <p class="token">
    {gameData.token}
  </p>
{/if}
<h1>
  {gameData.config.name}
  {#if gameData.host}
    <span class="host">Host</span>
  {/if}
</h1>
<p>{gameData.config.text}</p>

<!-- Start button if theres players in the game -->
{#if players.length > 0}
  <button on:click={doStart}>Start</button>
{/if}

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

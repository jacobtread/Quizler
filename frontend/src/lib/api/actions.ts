import { ClientMessage, HostAction } from "$lib/api/models";
import * as socket from "$lib/api/socket";
import type { GameData } from "$lib/pages/Game.svelte";
import { confirmDialog } from "$lib/stores/dialogStore";
import { setHome } from "$lib/stores/state";

export async function doKick(id: number) {
  try {
    await socket.send({
      ty: ClientMessage.Kick,
      id
    });
  } catch (e) {
    console.error("Error while attempting to kick", e);
  }
}

export async function doHostAction(action: HostAction): Promise<void> {
  try {
    await socket.send({
      ty: ClientMessage.HostAction,
      action
    });
  } catch (e) {
    console.error("Error while attempting host action", action, e);
  }
}

export async function setReady() {
  try {
    await socket.send({ ty: ClientMessage.Ready });
  } catch (e) {
    console.error("Error while attempting to ready", e);
  }
}

export async function leave(gameData: GameData) {
  const message = gameData.host
    ? "Are you sure you want to leave? Leaving will remove all other players from the game"
    : "Are you sure you want to leave?";

  const result = await confirmDialog("Confirm Leave", message);

  if (!result) return;

  // Kick self from game to leave
  doKick(gameData.id);

  // Take back to the home scren
  setHome();
}

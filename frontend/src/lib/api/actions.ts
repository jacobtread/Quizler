import { ClientMessage, HostAction } from "$lib/api/models";
import * as socket from "$lib/api/socket";

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

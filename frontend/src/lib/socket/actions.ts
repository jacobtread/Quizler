import { ClientMessage, HostAction } from "$lib/socket/models";
import * as socket from "$lib/socket";

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

async function doHostAction(action: HostAction): Promise<void> {
  try {
    await socket.send({
      ty: ClientMessage.HostAction,
      action
    });
  } catch (e) {
    console.error("Error while attempting host action", action, e);
  }
}

export function doHostStart(): Promise<void> {
  return doHostAction(HostAction.Start);
}

export function doHostCancel(): Promise<void> {
  return doHostAction(HostAction.Cancel);
}

export function doHostSkip(): Promise<void> {
  return doHostAction(HostAction.Skip);
}

export function doHostReset(): Promise<void> {
  return doHostAction(HostAction.Reset);
}

export async function setReady() {
  try {
    await socket.send({ ty: ClientMessage.Ready });
  } catch (e) {
    console.error("Error while attempting to ready", e);
  }
}

import * as socket from "$lib/socket";
import { ClientMessage, HostAction, ServerError } from "$lib/socket/models";

export async function doKick(id: number) {
  try {
    await socket.send({
      ty: ClientMessage.Kick,
      id
    });
  } catch (e) {
    const error = e as ServerError;
    console.error("Error while attempting to kick", error);
  }
}

async function doHostAction(action: HostAction): Promise<void> {
  try {
    await socket.send({
      ty: ClientMessage.HostAction,
      action
    });
  } catch (e) {
    const error = e as ServerError;
    console.error("Error while attempting host action", action, error);
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
    const error = e as ServerError;
    console.error("Error while attempting to ready", error);
  }
}

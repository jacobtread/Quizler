import { writable, type Writable } from "svelte/store";

type DialogCallback = (result: boolean) => void;

export const enum DialogType {
  Error,
  Confirm
}

type DialogState = { title: string; message: string } & (
  | { ty: DialogType.Error }
  | { ty: DialogType.Confirm; callback: DialogCallback }
);

// Store for state to manage selecting an image
export const dialogStore: Writable<DialogState | null> = writable(null);

export function consumeDialog(value: boolean) {
  dialogStore.update((state) => {
    if (state !== null && state.ty === DialogType.Confirm) {
      state.callback(value);
    }
    return null;
  });
}

export function confirmDialog(
  title: string,
  message: string
): Promise<boolean> {
  return new Promise((resolve) => {
    dialogStore.set({
      ty: DialogType.Confirm,
      title,
      message,
      callback: resolve
    });
  });
}

export function errorDialog(title: string, message: string) {
  dialogStore.set({
    ty: DialogType.Error,
    title,
    message
  });
}

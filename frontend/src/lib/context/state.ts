import type { AppStateStore } from "$lib/stores/state.svelte";
import { Context } from "runed";

const stateContext = new Context<AppStateStore>("AppStateStore");

export default stateContext;

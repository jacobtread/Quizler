import type { SocketStore } from "$lib/stores/socket.svelte";
import { Context } from "runed";

const socketContext = new Context<SocketStore>("SocketStore");

export default socketContext;

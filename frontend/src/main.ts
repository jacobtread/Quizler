import App from "./App.svelte";
import "./assets/app.scss";
import "$lib/socket";

const app = new App({
  target: document.getElementById("app") as Element
});

export default app;

import App from "./App.svelte";
import "./assets/app.scss";
import "$api/socket";

const app = new App({
  target: document.getElementById("app") as Element
});

export default app;

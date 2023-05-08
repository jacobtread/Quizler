import "$lib/socket";
import "$lib/assets/app.scss";
import App from "./App.svelte";

const app = new App({
  target: document.getElementById("app")
});

export default app;

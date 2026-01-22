import App from "./App.svelte";
import { mount } from "svelte";

console.log("[main.ts] Starting app mount...");
console.log("[main.ts] Target element:", document.getElementById("app"));

let app;

try {
  app = mount(App, {
    target: document.getElementById("app")!,
  });

  console.log("[main.ts] App mounted successfully:", app);
} catch (error) {
  console.error("[main.ts] ERROR mounting app:", error);
  throw error;
}

export default app;

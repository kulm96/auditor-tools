// Entry point for the auditor-tools shell.
// Uses TaskRouterStub to switch between the dashboard and File Conversion.

import { createTaskRouter } from "./router/TaskRouterStub";

function bootstrap() {
  const root = document.querySelector<HTMLDivElement>("#app");

  if (!root) {
    console.error("Failed to find #app root element");
    return;
  }

  // Initialize the minimal router; it will render the dashboard by default.
  createTaskRouter(root);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrap);
} else {
  bootstrap();
}


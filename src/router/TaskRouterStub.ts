// DEPRECATED PLACEHOLDER (PRD1): TaskRouterStub will be replaced by a full
// router as additional tasks are implemented in later PRDs.

import { renderDashboard } from "../dashboard";
import { renderFileConversionView } from "../tasks/file-conversion/FileConversionView";

export type TaskRoute = "dashboard" | "file-conversion";

export interface TaskRouter {
  /** Current route, managed internally by the stub router. */
  readonly current: TaskRoute;
  /** Navigate back to the main dashboard view. */
  goToDashboard(): void;
  /** Navigate to the File Conversion task home. */
  goToFileConversion(): void;
}

export function createTaskRouter(root: HTMLElement): TaskRouter {
  let current: TaskRoute = "dashboard";

  const api: TaskRouter = {
    get current() {
      return current;
    },
    goToDashboard() {
      current = "dashboard";
      renderDashboard(root);
      attachDashboardHandlers();
    },
    goToFileConversion() {
      current = "file-conversion";
      renderFileConversionView(root, {
        onBackToDashboard: () => api.goToDashboard(),
      });
    },
  };

  function attachDashboardHandlers() {
    const fileConversionCard = root.querySelector<HTMLElement>(
      '[data-task-id="file-conversion"]',
    );

    if (fileConversionCard) {
      fileConversionCard.addEventListener("click", () => {
        api.goToFileConversion();
      });
    }
  }

  // Initial render
  api.goToDashboard();

  return api;
}



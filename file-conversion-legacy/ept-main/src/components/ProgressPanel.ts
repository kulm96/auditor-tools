export class ProgressPanel {
  constructor() {}

  public render(): string {
    return `
      <div class="status-section">
        <div id="progress-section" class="progress-section" style="display: none;">
          <div class="progress-bar-container">
            <div id="progress-bar" class="progress-bar">
              <div id="progress-bar-fill" class="progress-bar-fill" style="width: 0%;"></div>
            </div>
            <div id="progress-text" class="progress-text">0 / 0</div>
          </div>
          <div id="task-category" class="task-category"></div>
        </div>
        <div id="status-message" class="status-message"></div>
      </div>
    `;
  }

  public update(
    current: number,
    total: number,
    taskCategory: string,
    isProcessing: boolean,
    statusMessage: string
  ): void {
    const progressSection = document.getElementById("progress-section");
    const statusMessageEl = document.getElementById("status-message");

    // Update visibility and message
    if (progressSection) {
      progressSection.style.display = isProcessing ? "block" : "none";
    }

    if (statusMessageEl) {
      statusMessageEl.innerHTML = statusMessage;
      statusMessageEl.style.display = statusMessage ? "block" : "none";
      // Preserve line breaks in the status message
      statusMessageEl.style.whiteSpace = "pre-line";
    }

    if (!isProcessing) return;

    // Update progress bars
    const progressBarFill = document.getElementById("progress-bar-fill");
    const progressText = document.getElementById("progress-text");
    const taskCategoryElement = document.getElementById("task-category");

    if (progressBarFill) {
      if (total > 0) {
        const percentage = Math.min(100, Math.round((current / total) * 100));
        progressBarFill.style.width = `${percentage}%`;
        progressBarFill.style.opacity = "1";
        progressBarFill.classList.remove("indeterminate");
      } else {
        // Show indeterminate progress (pulsing animation)
        progressBarFill.style.width = "100%";
        progressBarFill.style.opacity = "0.6";
        progressBarFill.classList.add("indeterminate");
      }
    }

    if (progressText) {
      if (total > 0) {
        progressText.textContent = `${current} / ${total}`;
      } else {
        progressText.textContent = "...";
      }
    }

    if (taskCategoryElement) {
      taskCategoryElement.textContent = taskCategory;
    }
  }
}


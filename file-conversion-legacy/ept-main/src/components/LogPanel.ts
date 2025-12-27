import { LogEntry } from "./AppState";

export class LogPanel {
  private lastRenderedCount: number = 0;
  private lastLogLevel: string = "";

  constructor() {}

  public render(): string {
    return `
      <div class="logs-section">
        <div class="logs-header">
          <div class="logs-header-left">
            <h2>Logs</h2>
            <select id="log-level-select" class="log-level-select">
              <option value="INFO">Info</option>
              <option value="WARNING" selected>Warning</option>
              <option value="ERROR">Error</option>
            </select>
          </div>
          <button id="clear-logs-button" class="clear-logs-button">Clear</button>
        </div>
        <div id="logs-panel" class="logs-panel"><div class="log-empty">No logs yet</div></div>
      </div>
    `;
  }

  public attachListeners(
    onClear: () => void,
    onLevelChange: (level: string) => void
  ): void {
    const clearLogsButton = document.getElementById("clear-logs-button");
    const logLevelSelect = document.getElementById(
      "log-level-select"
    ) as HTMLSelectElement;

    clearLogsButton?.addEventListener("click", () => {
      onClear();
      this.lastRenderedCount = 0; // Reset counter on clear
    });

    logLevelSelect?.addEventListener("change", (e) => {
      const target = e.target as HTMLSelectElement;
      onLevelChange(target.value);
    });
  }

  public update(logs: LogEntry[], selectedLogLevel: string): void {
    const logsPanel = document.getElementById("logs-panel");
    const logLevelSelect = document.getElementById(
      "log-level-select"
    ) as HTMLSelectElement;

    if (!logsPanel) return;

    // Sync select value if needed
    if (logLevelSelect && logLevelSelect.value !== selectedLogLevel) {
      logLevelSelect.value = selectedLogLevel;
    }

    // Check if we need a full re-render (log level changed or logs cleared/reset)
    const needsFullRender =
      selectedLogLevel !== this.lastLogLevel || logs.length < this.lastRenderedCount;

    this.lastLogLevel = selectedLogLevel;

    if (needsFullRender) {
      this.fullRender(logsPanel, logs, selectedLogLevel);
      return;
    }

    // If just adding new logs, append efficiently
    if (logs.length > this.lastRenderedCount) {
      this.appendLogs(logsPanel, logs, selectedLogLevel);
    }
  }

  private fullRender(
    container: HTMLElement,
    logs: LogEntry[],
    minLevel: string
  ): void {
    // Clear container
    container.innerHTML = "";
    this.lastRenderedCount = 0;

    const filteredLogs = this.filterLogsByLevel(logs, minLevel);

    if (filteredLogs.length === 0) {
      container.innerHTML = '<div class="log-empty">No logs yet</div>';
      this.lastRenderedCount = logs.length; // Sync count even if empty view
      return;
    }

    const fragment = document.createDocumentFragment();
    filteredLogs.forEach((log) => {
      const logElement = this.createLogElement(log);
      fragment.appendChild(logElement);
    });

    container.appendChild(fragment);
    container.scrollTop = container.scrollHeight;
    this.lastRenderedCount = logs.length;
  }

  private appendLogs(
    container: HTMLElement,
    logs: LogEntry[],
    minLevel: string
  ): void {
    // Remove "No logs yet" message if it exists
    const emptyMsg = container.querySelector(".log-empty");
    if (emptyMsg) {
      emptyMsg.remove();
    }

    const newLogs = logs.slice(this.lastRenderedCount);
    const fragment = document.createDocumentFragment();
    let hasNewVisibleLogs = false;

    newLogs.forEach((log) => {
      if (this.shouldShowLog(log, minLevel)) {
        const logElement = this.createLogElement(log);
        fragment.appendChild(logElement);
        hasNewVisibleLogs = true;
      }
    });

    if (hasNewVisibleLogs) {
      container.appendChild(fragment);
      container.scrollTop = container.scrollHeight;
    }

    this.lastRenderedCount = logs.length;
  }

  private createLogElement(log: LogEntry): HTMLElement {
    const div = document.createElement("div");
    div.className = `log-entry log-${log.level.toLowerCase()}`;

    const timestamp = document.createElement("span");
    timestamp.className = "log-timestamp";
    timestamp.textContent = new Date(log.timestamp).toLocaleTimeString();

    const level = document.createElement("span");
    level.className = "log-level";
    level.textContent = `[${log.level}]`;

    const message = document.createElement("span");
    message.className = "log-message";
    message.textContent = log.message;

    div.appendChild(timestamp);
    div.appendChild(level);
    div.appendChild(message);

    return div;
  }

  private shouldShowLog(log: LogEntry, minLevel: string): boolean {
    const levelHierarchy: { [key: string]: number } = {
      ERROR: 0,
      WARNING: 1,
      INFO: 2,
    };
    const minLevelIndex = levelHierarchy[minLevel] ?? 2;
    const logLevelIndex = levelHierarchy[log.level] ?? 2;
    return logLevelIndex <= minLevelIndex;
  }

  private filterLogsByLevel(logs: LogEntry[], minLevel: string): LogEntry[] {
    return logs.filter((log) => this.shouldShowLog(log, minLevel));
  }
}

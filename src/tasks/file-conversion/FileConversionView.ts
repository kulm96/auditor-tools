import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

export interface FileConversionViewOptions {
  onBackToDashboard: () => void;
}

/**
 * Renders the File Conversion task home view.
 *
 * NOTE: Backend integration is provided via a Tauri command that adapts to the
 * existing File Conversion logic from the legacy application per PRD1.
 */
export function renderFileConversionView(
  root: HTMLElement,
  options: FileConversionViewOptions,
) {
  root.innerHTML = `
    <div class="min-h-full flex flex-col bg-slate-950 text-slate-100">
      <header
        class="border-b border-slate-800 bg-slate-900/80 backdrop-blur flex items-center justify-between px-6 py-3"
      >
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="inline-flex items-center gap-1.5 rounded-full border border-slate-700 bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300 hover:border-slate-500 hover:text-slate-100 transition"
            data-nav="back-to-dashboard"
          >
            ← Dashboard
          </button>
          <div class="leading-tight">
            <div class="text-sm font-semibold tracking-tight">
              File Conversion
            </div>
            <div class="text-[11px] text-slate-400">
              Convert to Gemini‑friendly formats
            </div>
          </div>
        </div>
      </header>

      <main class="flex-1 flex flex-col px-6 py-6 gap-4">
        <section
          class="max-w-3xl rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900 to-slate-950/60 px-5 py-4 shadow-sm"
        >
          <h1 class="text-base font-semibold tracking-tight mb-1.5">
            Conversion job
          </h1>
          <p class="text-xs text-slate-400 mb-4">
            Select an input path and start a conversion job using the existing
            File Conversion engine.
          </p>

          <div class="flex flex-col gap-3">
            <label class="flex flex-col gap-1 text-xs text-slate-300">
              Input path (file or folder)
              <div class="flex items-center gap-2">
                <input
                  type="text"
                  data-field="input-path"
                  class="flex-1 rounded-md border border-slate-700 bg-slate-950/60 px-2 py-1.5 text-xs text-slate-100 placeholder:text-slate-600 focus:outline-none focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500"
                  placeholder="/path/to/evidence"
                />
                <button
                  type="button"
                  data-action="browse-folder"
                  class="inline-flex items-center gap-1.5 rounded-md border border-slate-700 bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300 hover:border-slate-500 hover:text-slate-100 transition whitespace-nowrap"
                >
                  Browse Folder
                </button>
                <button
                  type="button"
                  data-action="browse-file"
                  class="inline-flex items-center gap-1.5 rounded-md border border-slate-700 bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300 hover:border-slate-500 hover:text-slate-100 transition whitespace-nowrap"
                >
                  Browse File
                </button>
              </div>
            </label>

            <div class="flex items-center gap-2">
              <button
                type="button"
                data-action="start-conversion"
                class="inline-flex items-center gap-1.5 rounded-md bg-emerald-500 px-3 py-1.5 text-xs font-medium text-emerald-950 hover:bg-emerald-400 transition disabled:opacity-60 disabled:cursor-not-allowed"
              >
                Start conversion
              </button>
              <span
                class="text-[11px] text-slate-500"
              >
                Input: PDF, DOCX, XLSX, TXT · Output: Gemini‑friendly formats
              </span>
            </div>
          </div>
        </section>

        <section
          class="max-w-3xl rounded-xl border border-slate-900 bg-slate-950/80 px-5 py-3"
        >
          <div class="flex items-center justify-between mb-2">
            <h2 class="text-xs font-semibold text-slate-200">
              Activity log
            </h2>
            <div class="flex items-center gap-3">
              <label class="flex items-center gap-1.5 text-[11px] text-slate-400 cursor-pointer hover:text-slate-300">
                <input
                  type="checkbox"
                  data-field="debug-logs"
                  class="w-3.5 h-3.5 rounded border-slate-600 bg-slate-900 text-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:ring-offset-0"
                />
                <span>Debug logs</span>
              </label>
              <button
                type="button"
                data-action="clear-log"
                class="text-[11px] text-slate-500 hover:text-slate-300"
              >
                Clear
              </button>
            </div>
          </div>
          <pre
            data-panel="log"
            class="h-40 overflow-auto rounded-md bg-slate-950/90 border border-slate-900 text-[11px] text-slate-300 px-3 py-2 whitespace-pre-wrap"
          ></pre>
        </section>
      </main>
    </div>
  `;

  const backButton = root.querySelector<HTMLButtonElement>(
    '[data-nav="back-to-dashboard"]',
  );
  const startButton = root.querySelector<HTMLButtonElement>(
    '[data-action="start-conversion"]',
  );
  const clearButton = root.querySelector<HTMLButtonElement>(
    '[data-action="clear-log"]',
  );
  const inputField = root.querySelector<HTMLInputElement>(
    '[data-field="input-path"]',
  );
  const browseFolderButton = root.querySelector<HTMLButtonElement>(
    '[data-action="browse-folder"]',
  );
  const browseFileButton = root.querySelector<HTMLButtonElement>(
    '[data-action="browse-file"]',
  );
  const logPanel = root.querySelector<HTMLPreElement>(
    '[data-panel="log"]',
  );
  const debugLogsCheckbox = root.querySelector<HTMLInputElement>(
    '[data-field="debug-logs"]',
  );

  // Store all log entries
  interface LogEntry {
    level: string;
    message: string;
    timestamp: string;
  }
  const allLogs: LogEntry[] = [];
  let showDebugLogs = false;

  const appendLog = (message: string, level: string = "INFO") => {
    if (!logPanel) return;
    const now = new Date().toLocaleTimeString();
    const entry: LogEntry = {
      level,
      message,
      timestamp: now,
    };
    allLogs.push(entry);
    updateLogDisplay();
  };

  const formatTimestamp = (timestamp: string): string => {
    // Backend sends UTC timestamps like "2024-01-01 12:00:00.123 UTC"
    // Convert to local time for display, or use as-is if already formatted
    try {
      if (timestamp.includes("UTC")) {
        const date = new Date(timestamp.replace(" UTC", "Z"));
        return date.toLocaleTimeString();
      }
      return timestamp;
    } catch {
      return timestamp;
    }
  };

  const updateLogDisplay = () => {
    if (!logPanel) return;
    
    const logsToShow = showDebugLogs
      ? allLogs
      : allLogs.filter((log) => 
          log.level === "INFO" || 
          log.level === "WARNING" || 
          log.level === "ERROR"
        );
    
    logPanel.textContent = logsToShow
      .map((log) => {
        const formattedTime = formatTimestamp(log.timestamp);
        const levelPrefix = log.level === "ERROR" ? "❌" : 
                          log.level === "WARNING" ? "⚠️" : 
                          log.level === "DEBUG" ? "🔍" : "ℹ️";
        return `[${formattedTime}] ${levelPrefix} ${log.message}`;
      })
      .join("\n");
    
    // Auto-scroll to bottom to show latest logs
    logPanel.scrollTop = logPanel.scrollHeight;
  };

  backButton?.addEventListener("click", () => {
    options.onBackToDashboard();
  });

  clearButton?.addEventListener("click", () => {
    if (logPanel) {
      logPanel.textContent = "";
    }
    allLogs.length = 0;
  });

  debugLogsCheckbox?.addEventListener("change", (e) => {
    showDebugLogs = (e.target as HTMLInputElement).checked;
    updateLogDisplay();
  });

  // Listen to log-entry events from the backend for real-time updates
  let logListenerPromise: Promise<void> | null = null;
  
  const setupLogListener = () => {
    if (logListenerPromise) return logListenerPromise;
    
    logListenerPromise = listen<LogEntry>("log-entry", (event) => {
      const entry = event.payload;
      // Add log entry immediately for real-time display
      allLogs.push(entry);
      updateLogDisplay();
    }).then(() => {
      // Listener set up successfully
    }).catch((error) => {
      console.error("Failed to set up log listener:", error);
      appendLog(`Failed to set up log listener: ${error}`, "ERROR");
    });
    
    return logListenerPromise;
  };
  
  // Set up listener immediately when view is rendered
  setupLogListener();

  browseFolderButton?.addEventListener("click", async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string" && inputField) {
        inputField.value = selected;
      } else if (
        selected &&
        Array.isArray(selected) &&
        selected.length > 0 &&
        inputField
      ) {
        inputField.value = selected[0];
      }
    } catch (error) {
      appendLog(
        `Failed to open folder dialog: ${
          error instanceof Error ? error.message : String(error)
        }`,
      );
    }
  });

  browseFileButton?.addEventListener("click", async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        filters: [
          {
            name: "Archive Files",
            extensions: ["zip"],
          },
          {
            name: "All Files",
            extensions: ["*"],
          },
        ],
      });

      if (selected && typeof selected === "string" && inputField) {
        inputField.value = selected;
      } else if (
        selected &&
        Array.isArray(selected) &&
        selected.length > 0 &&
        inputField
      ) {
        inputField.value = selected[0];
      }
    } catch (error) {
      appendLog(
        `Failed to open file dialog: ${
          error instanceof Error ? error.message : String(error)
        }`,
      );
    }
  });

  startButton?.addEventListener("click", async () => {
    if (!inputField) return;
    const value = inputField.value.trim();
    if (!value) {
      appendLog("Please provide an input path before starting conversion.");
      return;
    }

    // Ensure log listener is set up before starting conversion
    await setupLogListener();

    startButton.disabled = true;
    // Don't manually log here - backend will send logs via events
    // Just clear any previous logs if needed, or let them accumulate

    try {
      // Adapter command; backend is responsible for delegating to the legacy
      // File Conversion logic without duplicating it.
      // All logs will come through the event listener in real-time
      await invoke("start_file_conversion", { inputPath: value });
      // Backend will send completion log via event
    } catch (error) {
      // Only log errors that aren't already logged by backend
      const errorMsg = error instanceof Error ? error.message : String(error);
      appendLog(
        `Conversion failed: ${errorMsg}`,
        "ERROR"
      );
    } finally {
      startButton.disabled = false;
    }
  });
}



import { invoke } from "@tauri-apps/api/core";

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
              <input
                type="text"
                data-field="input-path"
                class="rounded-md border border-slate-700 bg-slate-950/60 px-2 py-1.5 text-xs text-slate-100 placeholder:text-slate-600 focus:outline-none focus:ring-1 focus:ring-emerald-500 focus:border-emerald-500"
                placeholder="/path/to/evidence"
              />
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
            <button
              type="button"
              data-action="clear-log"
              class="text-[11px] text-slate-500 hover:text-slate-300"
            >
              Clear
            </button>
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
  const logPanel = root.querySelector<HTMLPreElement>(
    '[data-panel="log"]',
  );

  const appendLog = (message: string) => {
    if (!logPanel) return;
    const now = new Date().toLocaleTimeString();
    logPanel.textContent = `${logPanel.textContent ?? ""}[${now}] ${message}\n`;
    logPanel.scrollTop = logPanel.scrollHeight;
  };

  backButton?.addEventListener("click", () => {
    options.onBackToDashboard();
  });

  clearButton?.addEventListener("click", () => {
    if (logPanel) {
      logPanel.textContent = "";
    }
  });

  startButton?.addEventListener("click", async () => {
    if (!inputField) return;
    const value = inputField.value.trim();
    if (!value) {
      appendLog("Please provide an input path before starting conversion.");
      return;
    }

    startButton.disabled = true;
    appendLog(`Starting conversion for: ${value}`);

    try {
      // Adapter command; backend is responsible for delegating to the legacy
      // File Conversion logic without duplicating it.
      await invoke("start_file_conversion", { inputPath: value });
      appendLog("Conversion request completed.");
    } catch (error) {
      appendLog(
        `Conversion failed: ${
          error instanceof Error ? error.message : String(error)
        }`,
      );
    } finally {
      startButton.disabled = false;
    }
  });
}



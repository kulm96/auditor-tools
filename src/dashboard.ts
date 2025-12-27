// Dashboard layout for auditor-tools, based on layout1-dashboard-cards.html.
// Routing and card enable/disable behavior are provided by TaskRouterStub.

export function renderDashboard(root: HTMLElement) {
  root.innerHTML = `
    <div class="min-h-full flex flex-col bg-slate-950 text-slate-100">
      <header
        class="border-b border-slate-800 bg-slate-900/80 backdrop-blur flex items-center justify-between px-6 py-3"
      >
        <div class="flex items-center gap-2">
          <span
            class="inline-flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/30"
          >
            <span class="text-xs font-semibold">AT</span>
          </span>
          <div class="leading-tight">
            <div class="text-sm font-semibold tracking-tight">
              Auditor Tools
            </div>
            <div class="text-[11px] text-slate-400">
              Dashboard · PRD1
            </div>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <button
            class="inline-flex items-center gap-1.5 rounded-full border border-slate-700 bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300"
            type="button"
          >
            <span
              class="inline-flex h-4 w-4 items-center justify-center rounded-full bg-emerald-500/15 text-[9px] text-emerald-400 border border-emerald-500/40"
              >●</span
            >
            Session: Local
          </button>

          <button
            class="inline-flex items-center gap-1.5 rounded-full border border-slate-700 bg-slate-900/70 px-3 py-1.5 text-xs text-slate-300"
            type="button"
          >
            <span class="h-3.5 w-3.5 block">
              <!-- settings icon placeholder -->
            </span>
            Settings
          </button>
        </div>
      </header>

      <main class="flex-1 flex flex-col px-6 py-6 gap-6">
        <section class="flex flex-col gap-3">
          <div class="flex flex-wrap items-end justify-between gap-3">
            <div>
              <h1 class="text-xl font-semibold tracking-tight">
                Choose a tool to get started
              </h1>
              <p class="text-sm text-slate-400 max-w-2xl">
                Each card represents a task you can eventually launch from within the Tauri desktop app.
              </p>
            </div>
            <div class="flex items-center gap-2 text-xs text-slate-400">
              <span
                class="inline-flex h-6 items-center rounded-full border border-slate-700 bg-slate-900/80 px-2"
              >
                Layout 1 · Canonical dashboard
              </span>
            </div>
          </div>
        </section>

        <section
          class="grid gap-4 md:grid-cols-2 xl:grid-cols-3 auto-rows-[minmax(140px,1fr)]"
        >
          <!-- File Conversion (enabled in PRD1) -->
          <button
            class="group relative flex flex-col items-start rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900 to-slate-950/60 px-4 py-4 text-left shadow-sm hover:border-emerald-500/50 hover:shadow-emerald-500/10 transition"
            type="button"
            data-task-id="file-conversion"
          >
            <div
              class="mb-3 inline-flex items-center gap-2 rounded-full bg-emerald-500/10 px-2 py-1 text-[11px] text-emerald-300 border border-emerald-500/30"
            >
              <span
                class="h-1.5 w-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.7)]"
              ></span>
              File Conversion
            </div>
            <h2 class="text-sm font-medium">Convert to Gemini‑friendly formats</h2>
            <p class="mt-1.5 text-xs text-slate-400">
              Normalize mixed evidence sets into formats that work cleanly with
              Gemini (PDF, TXT, DOCX) while preserving structure where
              possible.
            </p>
            <div class="mt-auto flex w-full items-center justify-between pt-4">
              <span class="text-[11px] text-slate-500">
                Input: PDF, DOCX, XLSX, TXT
              </span>
              <span
                class="text-[11px] font-medium text-emerald-400 opacity-0 group-hover:opacity-100 transition"
              >
                Open →
              </span>
            </div>
          </button>

          <button
            class="group relative flex flex-col items-start rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900/40 to-slate-950/40 px-4 py-4 text-left opacity-40 cursor-not-allowed pointer-events-none"
            type="button"
            aria-disabled="true"
          >
            <div
              class="mb-3 inline-flex items-center gap-2 rounded-full bg-sky-500/10 px-2 py-1 text-[11px] text-sky-300 border border-sky-500/30"
            >
              <span class="h-1.5 w-1.5 rounded-full bg-sky-400"></span>
              DOCX/XLSX Unlocker
            </div>
            <h2 class="text-sm font-medium">Remove document protection</h2>
            <p class="mt-1.5 text-xs text-slate-400">
              Strip editing and form protections from DOCX and XLSX evidence
              files to make remediation and markup work easier.
            </p>
            <div class="mt-auto flex w-full items-center justify-between pt-4">
              <span class="text-[11px] text-slate-500">
                Input: DOCX, XLSX · Output: Unlocked copy
              </span>
            </div>
          </button>

          <button
            class="group relative flex flex-col items-start rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900/40 to-slate-950/40 px-4 py-4 text-left opacity-40 cursor-not-allowed pointer-events-none"
            type="button"
            aria-disabled="true"
          >
            <div
              class="mb-3 inline-flex items-center gap-2 rounded-full bg-fuchsia-500/10 px-2 py-1 text-[11px] text-fuchsia-300 border border-fuchsia-500/30"
            >
              <span class="h-1.5 w-1.5 rounded-full bg-fuchsia-400"></span>
              DOCX Checkbox Fixer
            </div>
            <h2 class="text-sm font-medium">Normalize form checkboxes</h2>
            <p class="mt-1.5 text-xs text-slate-400">
              Convert form-fillable checkboxes into simple, stable symbols that
              behave well across versions and viewers.
            </p>
            <div class="mt-auto flex w-full items-center justify-between pt-4">
              <span class="text-[11px] text-slate-500">
                Target: PCI DSS templates, internal forms
              </span>
            </div>
          </button>

          <button
            class="group relative flex flex-col items-start rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900/40 to-slate-950/40 px-4 py-4 text-left opacity-40 cursor-not-allowed pointer-events-none"
            type="button"
            aria-disabled="true"
          >
            <div
              class="mb-3 inline-flex items-center gap-2 rounded-full bg-amber-500/10 px-2 py-1 text-[11px] text-amber-300 border border-amber-500/30"
            >
              <span class="h-1.5 w-1.5 rounded-full bg-amber-400"></span>
              PCI DSS QA Tool
            </div>
            <h2 class="text-sm font-medium">Run structured QA on ROCs/AOCs</h2>
            <p class="mt-1.5 text-xs text-slate-400">
              Apply a repeatable QA checklist to PCI DSS ROCs and AOCs, track
              issues, and export findings for review.
            </p>
            <div class="mt-auto flex w-full items-center justify-between pt-4">
              <span class="text-[11px] text-slate-500">
                Focus: ROC consistency, scoping, evidence mapping
              </span>
            </div>
          </button>

          <button
            class="group relative flex flex-col items-start rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900 to-slate-950/60 px-4 py-4 text-left"
            type="button"
          >
            <div
              class="mb-3 inline-flex items-center gap-2 rounded-full bg-cyan-500/10 px-2 py-1 text-[11px] text-cyan-300 border border-cyan-500/30"
            >
              <span class="h-1.5 w-1.5 rounded-full bg-cyan-400"></span>
              Auto‑file‑opener
            </div>
            <h2 class="text-sm font-medium">Rapid evidence folder review</h2>
            <p class="mt-1.5 text-xs text-slate-400">
              Iterate through all files in an evidence folder using your
              default system viewers so you can review quickly and annotate.
            </p>
            <div class="mt-auto flex w-full items-center justify-between pt-4">
              <span class="text-[11px] text-slate-500">
                Backed by a Python helper script
              </span>
            </div>
          </button>

          <button
            class="group relative flex flex-col items-start rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900/40 to-slate-950/40 px-4 py-4 text-left md:col-span-2 xl:col-span-3 opacity-40 cursor-not-allowed pointer-events-none"
            type="button"
            aria-disabled="true"
          >
            <div class="flex w-full items-start justify-between gap-3">
              <div>
                <div
                  class="mb-3 inline-flex items-center gap-2 rounded-full bg-lime-500/10 px-2 py-1 text-[11px] text-lime-300 border border-lime-500/30"
                >
                  <span class="h-1.5 w-1.5 rounded-full bg-lime-400"></span>
                  PCI DSS ROC Checkbox Report
                </div>
                <h2 class="text-sm font-medium">
                  Export a complete checkbox state matrix
                </h2>
                <p class="mt-1.5 text-xs text-slate-400 max-w-2xl">
                  Parse ROC documents, detect checkbox states, and generate an
                  XLSX report summarizing all responses by requirement,
                  sub‑requirement, and testing procedure.
                </p>
              </div>
              <div
                class="hidden md:flex flex-col items-end gap-1 text-[11px] text-slate-500"
              >
                <span>Output: XLSX</span>
                <span>Use case: Peer review, QA, variance analysis</span>
              </div>
            </div>
            <div class="mt-4 flex flex-wrap items-center justify-between gap-3">
              <div class="flex flex-wrap gap-2 text-[11px] text-slate-400">
                <span
                  class="inline-flex items-center rounded-full bg-slate-900/80 px-2 py-1 border border-slate-700/80"
                  >Checkbox normalisation</span
                >
                <span
                  class="inline-flex items-center rounded-full bg-slate-900/80 px-2 py-1 border border-slate-700/80"
                  >ROC parsing</span
                >
                <span
                  class="inline-flex items-center rounded-full bg-slate-900/80 px-2 py-1 border border-slate-700/80"
                  >Excel report template</span
                >
              </div>
            </div>
          </button>
        </section>
      </main>
    </div>
  `;
}



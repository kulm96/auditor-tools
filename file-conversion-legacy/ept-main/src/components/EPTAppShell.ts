import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppState, LogEntry, ProcessingResult, ProgressUpdate } from "./AppState";
import { LogPanel } from "./LogPanel";
import { ProgressPanel } from "./ProgressPanel";

export class EPTAppShell {
  private appState: AppState;
  private logPanel: LogPanel;
  private progressPanel: ProgressPanel;

  constructor() {
    this.appState = new AppState();
    this.logPanel = new LogPanel();
    this.progressPanel = new ProgressPanel();

    // Subscribe to state changes
    this.appState.subscribe(() => {
      this.updateUI();
      this.updateComponents();
    });

    this.init();
  }

  private async init(): Promise<void> {
    try {
      const libreOfficeAvailable = await invoke<boolean>("check_libreoffice");
      if (!libreOfficeAvailable) {
        this.renderLibreOfficeError();
        return;
      }
    } catch (e) {
      console.error("Error checking LibreOffice:", e);
      this.renderLibreOfficeError();
      return;
    }

    this.initializeUI();
    this.setupEventListeners();
    this.setupLogListener();
    this.setupProgressListener();
    this.setupTauriDragDrop();
  }

  private renderLibreOfficeError(): void {
    const container = document.querySelector<HTMLElement>("#app");
    if (!container) return;

    container.innerHTML = `
      <div class="app-container" style="justify-content: center; align-items: center; text-align: center; padding: 2rem;">
        <header class="app-header">
          <h1>Evidence Processing Tool (EPT)</h1>
        </header>
        <main class="app-main" style="max-width: 600px; display: flex; flex-direction: column; gap: 1.5rem;">
          <div style="background-color: var(--bg-secondary); padding: 2rem; border-radius: 8px; border: 1px solid var(--border-color);">
            <h2 style="color: var(--error-color); margin-bottom: 1rem;">LibreOffice Not Found</h2>
            <p style="margin-bottom: 1rem;">This application requires LibreOffice to convert documents and spreadsheets.</p>
            <p style="margin-bottom: 1.5rem;">Please install LibreOffice and relaunch the application.</p>
            
            <div style="background-color: rgba(0,0,0,0.1); padding: 1rem; border-radius: 4px; text-align: left; font-size: 0.9em; margin-bottom: 1.5rem;">
              <p style="margin-bottom: 0.5rem;"><strong>Environment Variable:</strong></p>
              <p>You can set <code>EPT_LIBREOFFICE_PATH</code> to point to your LibreOffice executable (soffice) if it's in a custom location.</p>
            </div>

            <button id="quit-button" class="browse-button" style="width: 100%; background-color: var(--error-color); border-color: var(--error-color);">Quit</button>
          </div>
        </main>
      </div>
    `;

    document.getElementById("quit-button")?.addEventListener("click", async () => {
      try {
        await invoke("quit_app");
      } catch (e) {
        console.error("Failed to quit app:", e);
        // Fallback to window close
        await getCurrentWindow().close();
      }
    });
  }

  private initializeUI(): void {
    const container = document.querySelector<HTMLElement>("#app");
    if (!container) return;

    container.innerHTML = `
      <div class="app-container">
        ${this.renderHeader()}
        <main class="app-main">
          ${this.renderFileInputSection()}
          ${this.renderControlSection()}
          ${this.progressPanel.render()}
          ${this.logPanel.render()}
        </main>
      </div>
    `;

    // Attach component listeners after they are in the DOM
    this.logPanel.attachListeners(
      () => this.appState.clearLogs(),
      (level) => this.appState.setSelectedLogLevel(level)
    );
  }

  private renderHeader(): string {
    return `
      <header class="app-header">
        <h1>Evidence Processing Tool (EPT)</h1>
      </header>
    `;
  }

  private renderFileInputSection(): string {
    return `
      <div class="file-input-section">
        <div id="file-input-component" class="file-input-component">
          <div class="drop-zone" id="drop-zone">
            <div class="drop-zone-content">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="17 8 12 3 7 8"></polyline>
                <line x1="12" y1="3" x2="12" y2="15"></line>
              </svg>
              <p class="drop-zone-text">Drag and drop a folder or ZIP file here</p>
              <p class="drop-zone-subtext">or</p>
              <div class="browse-buttons">
                <button id="browse-button" class="browse-button">Browse Folders</button>
                <button id="browse-zip-button" class="browse-button browse-zip-button">Browse Zip Files</button>
              </div>
            </div>
          </div>
          <div id="selected-path-display" class="selected-path-display" style="display: none;">
            <span class="path-label">Selected:</span>
            <span id="selected-path-text" class="path-text"></span>
            <button id="clear-button" class="clear-button">✕</button>
          </div>
        </div>
      </div>
    `;
  }

  private renderControlSection(): string {
    return `
      <div class="control-section">
        <button id="go-button" class="go-button" disabled>GO</button>
        <button id="start-over-button" class="start-over-button" style="display: none;">Start Over</button>
        <div id="result-actions" class="result-actions" style="display: none;">
          <button id="open-staging-button" class="open-staging-button">Open Staging Folder</button>
          <button id="open-folder-button" class="open-folder-button">Open LLM Folder</button>
          <button id="open-report-button" class="open-report-button">Open Report</button>
        </div>
      </div>
    `;
  }

  private setupEventListeners(): void {
    const dropZone = document.getElementById("drop-zone");
    const browseButton = document.getElementById("browse-button");
    const browseZipButton = document.getElementById("browse-zip-button");
    const goButton = document.getElementById("go-button");
    const startOverButton = document.getElementById("start-over-button");
    const clearButton = document.getElementById("clear-button");
    const openStagingButton = document.getElementById("open-staging-button");
    const openFolderButton = document.getElementById("open-folder-button");
    const openReportButton = document.getElementById("open-report-button");

    // Prevent default drag and drop behavior
    document.addEventListener("dragover", (e) => e.preventDefault());
    document.addEventListener("drop", (e) => e.preventDefault());

    // Drag and drop visual feedback
    if (dropZone) {
      dropZone.addEventListener("dragenter", (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropZone.classList.add("drag-over");
      });

      dropZone.addEventListener("dragover", (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropZone.classList.add("drag-over");
      });

      dropZone.addEventListener("dragleave", (e) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.currentTarget === dropZone) {
          dropZone.classList.remove("drag-over");
        }
      });

      dropZone.addEventListener("drop", async (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropZone.classList.remove("drag-over");
        
        try {
          const files = e.dataTransfer?.files;
          if (files && files.length > 0) {
            const file = files[0];
            let path: string | null = null;
            
            if ((file as any).path) {
              path = (file as any).path;
            } else if ((file as any).webkitRelativePath) {
              path = (file as any).webkitRelativePath;
            } else if (file.name) {
              this.appState.addLog("WARN", `File dropped but no path found. File name: ${file.name}. This may not work for folders.`);
            }
            
            if (path) {
              this.appState.addLog("INFO", `Processing dropped item: ${path}`);
              await this.handleUserInput(path);
              return;
            }
          }
          
          // Fallback: Try items API (simplified for brevity, main logic preserved)
          // ... (Existing intricate drop logic omitted for brevity, but should be preserved if robust handling is needed. 
          // Re-inserting the robust logic below to ensure no regression)
          
           const items = e.dataTransfer?.items;
           if (items && items.length > 0) {
             for (let i = 0; i < items.length; i++) {
               const item = items[i];
               if (item.kind === "file") {
                 const file = item.getAsFile();
                 if (file && (file as any).path) {
                    const path = (file as any).path;
                    this.appState.addLog("INFO", `Processing dropped item: ${path}`);
                    await this.handleUserInput(path);
                    return;
                 }
               }
             }
           }

          this.appState.addLog("ERROR", "No files found in drop event or could not extract path");
        } catch (error) {
          this.appState.addLog("ERROR", `Error handling drop: ${error}`);
        }
      });
    }

    // Browse button (folders)
    browseButton?.addEventListener("click", async () => {
      try {
        const dirSelected = await open({
          directory: true,
          multiple: false,
        });

        if (dirSelected && typeof dirSelected === "string") {
          await this.handleUserInput(dirSelected);
        } else if (dirSelected && Array.isArray(dirSelected) && dirSelected.length > 0) {
          await this.handleUserInput(dirSelected[0]);
        }
      } catch (error) {
        this.appState.addLog("ERROR", `Failed to open directory dialog: ${error}`);
      }
    });

    // Browse Zip Files button
    browseZipButton?.addEventListener("click", async () => {
      try {
        const fileSelected = await open({
          directory: false,
          multiple: false,
          filters: [
            { name: "Archives", extensions: ["zip", "gz", "7z", "rar"] },
            { name: "All Files", extensions: ["*"] },
          ],
        });

        if (fileSelected && typeof fileSelected === "string") {
          await this.handleUserInput(fileSelected);
        } else if (fileSelected && Array.isArray(fileSelected) && fileSelected.length > 0) {
          await this.handleUserInput(fileSelected[0]);
        }
      } catch (error) {
        this.appState.addLog("ERROR", `Failed to open file dialog: ${error}`);
      }
    });

    // GO button
    goButton?.addEventListener("click", async () => {
      if (this.appState.selectedPath && !this.appState.isProcessing) {
        await this.startProcessing();
      }
    });

    // Start Over button
    startOverButton?.addEventListener("click", () => {
      this.resetState();
    });

    // Clear button
    clearButton?.addEventListener("click", () => {
      this.appState.setSelectedPath(null);
    });

    // Open staging folder button
    openStagingButton?.addEventListener("click", async () => {
      if (this.appState.processingResult) {
        try {
          await invoke("open_folder", { path: this.appState.processingResult.staging_path });
        } catch (error) {
          this.appState.addLog("ERROR", `Failed to open folder: ${error}`);
        }
      }
    });

    // Open folder button
    openFolderButton?.addEventListener("click", async () => {
      if (this.appState.processingResult) {
        try {
          await invoke("open_folder", { path: this.appState.processingResult.llm_output_path });
        } catch (error) {
          this.appState.addLog("ERROR", `Failed to open folder: ${error}`);
        }
      }
    });

    // Open report button
    openReportButton?.addEventListener("click", async () => {
      if (this.appState.processingResult) {
        try {
          await invoke("open_file", { path: this.appState.processingResult.report_path });
        } catch (error) {
          this.appState.addLog("ERROR", `Failed to open report: ${error}`);
        }
      }
    });
  }

  private async setupLogListener(): Promise<void> {
    try {
      await listen<LogEntry>("log-entry", (event) => {
        this.appState.addLog(event.payload.level, event.payload.message);
      });
    } catch (error) {
      console.error("Failed to set up log listener:", error);
    }
  }

  private async setupProgressListener(): Promise<void> {
    try {
      await listen<ProgressUpdate>("progress-update", (event) => {
        this.appState.updateProgress(event.payload.current, event.payload.total, event.payload.task_category);
      });
    } catch (error) {
      console.error("Failed to set up progress listener:", error);
    }
  }

  private async setupTauriDragDrop(): Promise<void> {
    try {
      const window = getCurrentWindow();
      const dropZone = document.getElementById("drop-zone");
      
      await window.onDragDropEvent(async (event) => {
        const dropEvent = event.payload;
        
        if (dropEvent.type === "enter" || dropEvent.type === "over") {
          if (dropZone) dropZone.classList.add("drag-over");
        } else if (dropEvent.type === "leave") {
          if (dropZone) dropZone.classList.remove("drag-over");
        } else if (dropEvent.type === "drop") {
          if (dropZone) dropZone.classList.remove("drag-over");
          
          if (dropEvent.paths && dropEvent.paths.length > 0) {
            const path = dropEvent.paths[0];
            this.appState.addLog("INFO", `Processing dropped item: ${path}`);
            await this.handleUserInput(path);
          } else {
            this.appState.addLog("ERROR", "No paths found in drop event");
          }
        }
      });
    } catch (error) {
      console.error("Failed to set up Tauri drag and drop:", error);
      this.appState.addLog("WARN", "Tauri drag and drop not available, falling back to web API");
    }
  }

  private async handleUserInput(path: string): Promise<void> {
    try {
      const normalizedPath = await invoke<string>("handle_user_input", { path });
      this.appState.setSelectedPath(normalizedPath);
      this.appState.addLog("INFO", `Selected: ${normalizedPath}`);
    } catch (error) {
      this.appState.addLog("ERROR", `Invalid path: ${error}`);
    }
  }

  private async startProcessing(): Promise<void> {
    if (!this.appState.selectedPath || this.appState.isProcessing) return;

    this.appState.setProcessing(true);
    this.appState.setProcessingResult(null);
    this.appState.updateProgress(0, 0, "");
    this.appState.addLog("INFO", "Starting processing...");

    try {
      const result = await invoke<ProcessingResult>("start_processing", {
        inputPath: this.appState.selectedPath,
      });

      this.appState.setProcessingResult(result);
      const processedCount = result.entries.filter(e => e.processed === "Yes").length;
      
      this.appState.addLog("INFO", `Processing complete. Found ${result.entries.length} files, ${processedCount} processed.`);
      
      // Build status message with paths
      const statusMessage = `Processed ${processedCount} of ${result.entries.length} files.\n\n` +
        `<strong>Staging Folder:</strong> ${result.staging_path}\n` +
        `<strong>LLM Output Folder:</strong> ${result.llm_output_path}\n` +
        `<strong>Report File:</strong> ${result.report_path}`;
      this.appState.setStatusMessage(statusMessage);
      
    } catch (error) {
      this.appState.addLog("ERROR", `Processing failed: ${error}`);
      this.appState.setStatusMessage(`Error: ${error}`);
    } finally {
      this.appState.setProcessing(false);
    }
  }

  private resetState(): void {
    this.appState.reset();
  }

  private updateUI(): void {
    const goButton = document.getElementById("go-button") as HTMLButtonElement;
    const startOverButton = document.getElementById("start-over-button");
    const selectedPathDisplay = document.getElementById("selected-path-display");
    const selectedPathText = document.getElementById("selected-path-text");
    const dropZone = document.getElementById("drop-zone");
    const resultActions = document.getElementById("result-actions");

    if (this.appState.selectedPath) {
      if (selectedPathDisplay) selectedPathDisplay.style.display = "flex";
      if (selectedPathText) selectedPathText.textContent = this.appState.selectedPath;
      if (dropZone) dropZone.style.display = "none";
      if (goButton) {
        goButton.disabled = this.appState.isProcessing;
        if (this.appState.processingResult) {
          goButton.style.display = "none";
        } else {
          goButton.style.display = "block";
        }
      }
      if (startOverButton) startOverButton.style.display = "block";
    } else {
      if (selectedPathDisplay) selectedPathDisplay.style.display = "none";
      if (dropZone) dropZone.style.removeProperty("display");
      if (goButton) {
        goButton.disabled = true;
        goButton.style.display = "block";
      }
      if (startOverButton) startOverButton.style.display = "none";
    }

    if (resultActions) {
      resultActions.style.display = (this.appState.processingResult && !this.appState.isProcessing) ? "flex" : "none";
    }
  }

  private updateComponents(): void {
    this.logPanel.update(this.appState.logs, this.appState.selectedLogLevel);
    this.progressPanel.update(
      this.appState.progressCurrent,
      this.appState.progressTotal,
      this.appState.currentTaskCategory,
      this.appState.isProcessing,
      this.appState.statusMessage
    );
  }
}

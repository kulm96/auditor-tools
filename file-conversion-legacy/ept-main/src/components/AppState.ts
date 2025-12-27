export interface LogEntry {
  level: string;
  message: string;
  timestamp: string;
}

export interface ReportModel {
  file_name: string;
  sha512: string | null;
  processed: string;
  skip_reason: string | null;
  relative_path: string;
  file_type: string;
  file_size_bytes: number;
  file_size_human: string;
  last_modified: string;
  created_time: string;
}

export interface ProcessingResult {
  entries: ReportModel[];
  staging_path: string;
  llm_output_path: string;
  report_path: string;
}

export interface ProgressUpdate {
  current: number;
  total: number;
  task_category: string;
}

type Listener = () => void;

export class AppState {
  // State properties
  public selectedPath: string | null = null;
  public isProcessing: boolean = false;
  public logs: LogEntry[] = [];
  public processingResult: ProcessingResult | null = null;
  public selectedLogLevel: string = "WARNING";
  public progressCurrent: number = 0;
  public progressTotal: number = 0;
  public currentTaskCategory: string = "";
  public statusMessage: string = "";

  private listeners: Listener[] = [];

  constructor() {}

  // State mutation methods
  public setSelectedPath(path: string | null) {
    this.selectedPath = path;
    this.notify();
  }

  public setProcessing(isProcessing: boolean) {
    this.isProcessing = isProcessing;
    this.notify();
  }

  public setProcessingResult(result: ProcessingResult | null) {
    this.processingResult = result;
    this.notify();
  }

  public setSelectedLogLevel(level: string) {
    this.selectedLogLevel = level;
    this.notify(); // UI needs to re-render logs
  }

  public updateProgress(current: number, total: number, taskCategory: string) {
    this.progressCurrent = current;
    this.progressTotal = total;
    this.currentTaskCategory = taskCategory;
    this.notify();
  }

  public setStatusMessage(message: string) {
    this.statusMessage = message;
    this.notify();
  }

  public addLog(level: string, message: string) {
    const timestamp = new Date().toISOString();
    const entry: LogEntry = {
      level,
      message,
      timestamp,
    };
    this.logs.push(entry);
    this.notify();
  }

  public clearLogs() {
    this.logs = [];
    this.notify();
  }

  public reset() {
    this.selectedPath = null;
    this.isProcessing = false;
    this.logs = [];
    this.processingResult = null;
    this.selectedLogLevel = "WARNING";
    this.progressCurrent = 0;
    this.progressTotal = 0;
    this.currentTaskCategory = "";
    this.statusMessage = "";
    this.notify();
  }

  // Subscription mechanism
  public subscribe(listener: Listener): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  private notify() {
    this.listeners.forEach((listener) => listener());
  }
}


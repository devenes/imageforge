import type {
  CompressionResult,
  CompressionSettings,
  CompressionStatus,
  ImageInfo,
  ProgressUpdate,
  QueueItem,
} from '$lib/types';

class AppState {
  queue = $state<QueueItem[]>([]);
  settings = $state<CompressionSettings>({
    preset: 'balanced',
    outputMode: 'same_format',
    preserveMetadata: true,
    preserveColorProfile: true,
    customOutputDir: null,
    filenameSuffix: '-compressed',
  });
  isProcessing = $state<boolean>(false);
  progress = $state<ProgressUpdate | null>(null);
  results = $state<CompressionResult[]>([]);
  hasCompletedBatch = $state<boolean>(false);
  showPreferences = $state<boolean>(false);
  errorBanner = $state<string | null>(null);

  get isQueueEmpty() {
    return this.queue.length === 0;
  }

  get totalOriginalBytes() {
    return this.queue.reduce((acc, item) => acc + item.info.bytes, 0);
  }

  get totalOutputBytes() {
    return this.results.reduce((acc, res) => {
      // If completed with savings, use outputBytes; if no savings, use original
      if (res.status === 'COMPLETED') {
        return acc + res.outputBytes;
      }
      return acc + res.inputBytes;
    }, 0);
  }

  get totalSavedBytes() {
    return this.results.reduce((acc, res) => {
      if (res.status === 'COMPLETED') {
        return acc + res.savedBytes;
      }
      return acc;
    }, 0);
  }

  get overallReductionPercent() {
    if (this.totalOriginalBytes === 0) return 0;
    return (this.totalSavedBytes / this.totalOriginalBytes) * 100;
  }

  get completedCount() {
    return this.results.filter(
      (r) => r.status === 'COMPLETED' || r.status === 'NO_SAVINGS'
    ).length;
  }

  addFiles(infos: ImageInfo[]) {
    const existingPaths = new Set(this.queue.map((item) => item.info.path));
    const newItems: QueueItem[] = [];

    for (const info of infos) {
      if (!existingPaths.has(info.path)) {
        newItems.push({
          info,
          status: 'QUEUED',
        });
        existingPaths.add(info.path);
      }
    }

    if (newItems.length > 0) {
      this.queue = [...this.queue, ...newItems];
      this.hasCompletedBatch = false;
      this.results = [];
    }
  }

  removeItem(id: string) {
    this.queue = this.queue.filter((item) => item.info.id !== id);
    if (this.queue.length === 0) {
      this.hasCompletedBatch = false;
      this.results = [];
    }
  }

  clearQueue() {
    if (this.isProcessing) return;
    this.queue = [];
    this.results = [];
    this.hasCompletedBatch = false;
    this.progress = null;
  }

  updateItemStatus(id: string, status: CompressionStatus, result?: CompressionResult) {
    const item = this.queue.find((q) => q.info.id === id);
    if (item) {
      item.status = status;
      if (result) {
        item.result = result;
      }
    }
    if (result) {
      // Add or replace in results array
      const idx = this.results.findIndex((r) => r.id === id);
      if (idx >= 0) {
        this.results[idx] = result;
      } else {
        this.results.push(result);
      }
    }
  }

  finishBatch(allResults: CompressionResult[]) {
    this.isProcessing = false;
    this.hasCompletedBatch = true;
    this.progress = null;
    this.results = allResults;
    for (const res of allResults) {
      const item = this.queue.find((q) => q.info.id === res.id);
      if (item) {
        item.status = res.status;
        item.result = res;
      }
    }
  }

  cancelBatch() {
    this.isProcessing = false;
    this.progress = null;
  }

  resetForNewBatch() {
    this.queue = [];
    this.results = [];
    this.hasCompletedBatch = false;
    this.progress = null;
  }

  setError(msg: string | null) {
    this.errorBanner = msg;
  }
}

export const appState = new AppState();

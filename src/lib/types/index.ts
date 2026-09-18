export type ImageFormat = 'jpg' | 'png' | 'webp' | 'heic';

export interface ImageInfo {
  id: string;
  path: string;
  filename: string;
  format: ImageFormat;
  width: number;
  height: number;
  bytes: number;
  orientation: number | null;
  hasAlpha: boolean;
  colorProfile: string | null;
  metadataAvailable: boolean;
}

export type CompressionPreset = 'best_quality' | 'balanced' | 'smallest';

export type OutputMode = 'same_format' | 'convert_to_jpg';

export interface CompressionSettings {
  preset: CompressionPreset;
  outputMode: OutputMode;
  preserveMetadata: boolean;
  preserveColorProfile: boolean;
  customOutputDir: string | null;
  filenameSuffix: string | null;
}

export type CompressionStatus =
  | 'QUEUED'
  | 'PROCESSING'
  | 'COMPLETED'
  | 'SKIPPED'
  | 'NO_SAVINGS'
  | 'FAILED';

export interface CompressionResult {
  id: string;
  inputBytes: number;
  outputBytes: number;
  savedBytes: number;
  savedPercent: number;
  inputWidth: number;
  inputHeight: number;
  outputWidth: number;
  outputHeight: number;
  inputFormat: ImageFormat;
  outputFormat: ImageFormat;
  outputPath: string | null;
  durationMs: number;
  status: CompressionStatus;
  error: string | null;
  colorProfilePreserved: boolean;
}

export interface QueueItem {
  info: ImageInfo;
  status: CompressionStatus;
  result?: CompressionResult;
}

export interface ProgressUpdate {
  currentIndex: number;
  totalCount: number;
  currentFilename: string;
  stage: string;
}

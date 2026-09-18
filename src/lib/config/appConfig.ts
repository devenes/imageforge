export interface AppConfig {
  name: string;
  version: string;
  tagline: string;
  description: string;
  supportedInputFormats: string[];
  supportedOutputFormats: string[];
  conversionTarget: string;
}

export const APP_CONFIG: AppConfig = {
  name: 'ImageForge',
  version: '0.1.0',
  tagline: 'Make your images smaller without changing their dimensions.',
  description: 'A local-first macOS image compressor and JPG converter.',
  supportedInputFormats: ['JPG', 'PNG', 'WebP', 'HEIC'],
  supportedOutputFormats: ['JPG', 'PNG', 'WebP', 'HEIC'],
  conversionTarget: 'JPG',
};

export interface StorageAssistantStatus {
  installed: boolean;
  verified: boolean;
  busy: boolean;
  model: string;
  quantization: string;
  runtime: string;
  modelBytes: number;
  expectedBytes: number;
  modelPath: string;
  license: string;
  privacyNote: string;
}

export interface AssistantDownloadProgress {
  phase: "connecting" | "downloading" | "verifying" | "ready" | string;
  downloadedBytes: number;
  totalBytes: number;
}

export interface AssistantAnnotation {
  findingId: string;
  suggestedName: string;
  group: string;
  explanation: string;
  confidence: number;
}

export interface StorageAssistantReport {
  scanId: string;
  generatedAt: string;
  model: string;
  summary: string;
  observations: string[];
  annotations: AssistantAnnotation[];
  advisoryOnly: boolean;
}

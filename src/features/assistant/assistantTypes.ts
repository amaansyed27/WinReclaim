export interface StorageAssistantStatus {
  available: boolean;
  busy: boolean;
  provider: string;
  model: string;
  privacyNote: string;
}

export interface StorageAssistantReport {
  scanId: string;
  generatedAt: string;
  model: string;
  summary: string;
  observations: string[];
  advisoryOnly: boolean;
}

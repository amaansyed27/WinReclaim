export interface AppDataSummary {
  root: string;
  snapshotCount: number;
  receiptCount: number;
  vaultEntryCount: number;
  vaultBytes: number;
}

export interface AppDataMutation {
  removedEntries: number;
  removedBytes: number;
  includedRestoreFiles: boolean;
}

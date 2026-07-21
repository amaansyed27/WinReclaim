mod download;
mod inference;
mod prompt;

use crate::app_data;
use crate::domain::{ScanReport, StorageAssistantReport, StorageAssistantStatus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

pub const MODEL_NAME: &str = "WinReclaim Storage Assistant";
pub const BASE_MODEL: &str = "Qwen3.5-2B";
pub const QUANTIZATION: &str = "Q4_K_M";
pub const MODEL_FILE: &str = "Qwen3.5-2B-Q4_K_M.gguf";
pub const MODEL_SHA256: &str = "84aeb7fe40e7b833d71303d7f1b9f9c1991b931b5dbd214e0aa48d56a0af1f85";
pub const MODEL_EXPECTED_BYTES: u64 = 1_400_000_000;
pub const MODEL_URL: &str = "https://huggingface.co/bartowski/Qwen_Qwen3.5-2B-GGUF/resolve/915a52556175c333102d04f996380950d35155d9/Qwen_Qwen3.5-2B-Q4_K_M.gguf?download=true";
const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelManifest {
    pub base_model: String,
    pub quantization: String,
    pub sha256: String,
    pub bytes: u64,
    pub source: String,
    pub license: String,
    pub installed_at: String,
}

pub fn status(busy: bool) -> StorageAssistantStatus {
    let model_path = model_path();
    let manifest = read_manifest();
    let file_bytes = fs::metadata(&model_path).map(|metadata| metadata.len()).unwrap_or(0);
    let verified = manifest.as_ref().is_some_and(|manifest| {
        manifest.sha256.eq_ignore_ascii_case(MODEL_SHA256)
            && manifest.bytes == file_bytes
            && file_bytes > 0
    });

    StorageAssistantStatus {
        installed: model_path.is_file() && manifest.is_some(),
        verified,
        busy,
        model: format!("{BASE_MODEL} {QUANTIZATION}"),
        quantization: QUANTIZATION.to_string(),
        runtime: "llama.cpp CPU runtime embedded in WinReclaim".to_string(),
        model_bytes: file_bytes,
        expected_bytes: manifest
            .as_ref()
            .map(|manifest| manifest.bytes)
            .unwrap_or(MODEL_EXPECTED_BYTES),
        model_path: model_path.to_string_lossy().to_string(),
        license: "Apache-2.0".to_string(),
        privacy_note: "The model runs locally and receives only the completed scan report. It cannot enable, select or execute cleanup actions."
            .to_string(),
    }
}

pub fn install(app: &AppHandle) -> Result<StorageAssistantStatus> {
    download::install(app)?;
    Ok(status(false))
}

pub fn uninstall() -> Result<StorageAssistantStatus> {
    let root = model_root();
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    Ok(status(false))
}

pub fn analyze(report: &ScanReport) -> Result<StorageAssistantReport> {
    let prompt = prompt::build_report_prompt(report)?;
    let output = inference::generate(&model_path(), &prompt, 900)?;
    inference::parse_report(report, &output)
}

pub(crate) fn model_root() -> PathBuf {
    app_data::app_root().join("models").join("storage-assistant")
}

pub(crate) fn model_path() -> PathBuf {
    model_root().join(MODEL_FILE)
}

pub(crate) fn manifest_path() -> PathBuf {
    model_root().join(MANIFEST_FILE)
}

fn read_manifest() -> Option<ModelManifest> {
    let content = fs::read_to_string(manifest_path()).ok()?;
    serde_json::from_str(&content).ok()
}

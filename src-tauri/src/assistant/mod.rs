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

pub const RUNTIME_NAME: &str = "llama.cpp CPU runtime";
pub const RUNTIME_TAG: &str = "b9993";
pub const RUNTIME_ASSET: &str = "llama-b9993-bin-win-cpu-x64.zip";
pub const RUNTIME_RELEASE_API: &str =
    "https://api.github.com/repos/ggml-org/llama.cpp/releases/tags/b9993";
pub const RUNTIME_EXPECTED_BYTES: u64 = 19_500_000;
pub const RUNTIME_EXE: &str = "llama-cli.exe";

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
    #[serde(default)]
    pub runtime_tag: String,
    #[serde(default)]
    pub runtime_asset: String,
    #[serde(default)]
    pub runtime_sha256: String,
    #[serde(default)]
    pub runtime_bytes: u64,
}

pub fn status(busy: bool) -> StorageAssistantStatus {
    let model_path = model_path();
    let runtime_path = runtime_path();
    let manifest = read_manifest();
    let file_bytes = fs::metadata(&model_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    let model_verified = manifest.as_ref().is_some_and(|manifest| {
        manifest.sha256.eq_ignore_ascii_case(MODEL_SHA256)
            && manifest.bytes == file_bytes
            && file_bytes > 0
    });
    let runtime_verified = manifest.as_ref().is_some_and(|manifest| {
        manifest.runtime_tag == RUNTIME_TAG
            && manifest.runtime_asset == RUNTIME_ASSET
            && manifest.runtime_sha256.len() == 64
            && manifest.runtime_bytes > 0
            && runtime_path.is_file()
    });

    StorageAssistantStatus {
        installed: model_path.is_file() && runtime_path.is_file() && manifest.is_some(),
        verified: model_verified && runtime_verified,
        busy,
        model: format!("{BASE_MODEL} {QUANTIZATION}"),
        quantization: QUANTIZATION.to_string(),
        runtime: format!(
            "{RUNTIME_NAME} {RUNTIME_TAG} sidecar, downloaded only with the assistant"
        ),
        model_bytes: file_bytes,
        expected_bytes: manifest
            .as_ref()
            .map(|manifest| manifest.bytes)
            .unwrap_or(MODEL_EXPECTED_BYTES),
        model_path: model_path.to_string_lossy().to_string(),
        license: "Apache-2.0 model · MIT runtime".to_string(),
        privacy_note: "The model and runtime are optional local downloads. The model receives only the completed scan report and cannot enable, select or execute cleanup actions."
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
    let output = inference::generate(&runtime_path(), &model_path(), &prompt, 1_400)?;
    inference::parse_report(report, &output)
}

pub(crate) fn model_root() -> PathBuf {
    app_data::app_root()
        .join("models")
        .join("storage-assistant")
}

pub(crate) fn model_path() -> PathBuf {
    model_root().join(MODEL_FILE)
}

pub(crate) fn runtime_root() -> PathBuf {
    model_root().join("runtime").join(RUNTIME_TAG)
}

pub(crate) fn runtime_path() -> PathBuf {
    runtime_root().join(RUNTIME_EXE)
}

pub(crate) fn manifest_path() -> PathBuf {
    model_root().join(MANIFEST_FILE)
}

fn read_manifest() -> Option<ModelManifest> {
    let content = fs::read_to_string(manifest_path()).ok()?;
    serde_json::from_str(&content).ok()
}

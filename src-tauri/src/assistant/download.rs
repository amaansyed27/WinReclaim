use super::{
    manifest_path, model_path, model_root, ModelManifest, BASE_MODEL, MODEL_EXPECTED_BYTES,
    MODEL_SHA256, MODEL_URL, QUANTIZATION,
};
use crate::domain::AssistantDownloadProgress;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const BUFFER_BYTES: usize = 1024 * 1024;
const PROGRESS_INTERVAL_BYTES: u64 = 8 * 1024 * 1024;

pub fn install(app: &AppHandle) -> Result<()> {
    if super::status(false).verified {
        emit_progress(app, "ready", 0, 0);
        return Ok(());
    }

    let root = model_root();
    fs::create_dir_all(&root)?;
    let target = model_path();
    let partial = root.join(format!("{}.partial", super::MODEL_FILE));
    let _ = fs::remove_file(&partial);
    if target.exists() {
        fs::remove_file(&target).context("Unable to replace the unverified assistant model")?;
    }

    emit_progress(app, "connecting", 0, MODEL_EXPECTED_BYTES);
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(60 * 60))
        .user_agent("WinReclaim/1.1 storage-assistant")
        .build()?;
    let mut response = client
        .get(MODEL_URL)
        .send()
        .context("Unable to start the Storage Assistant download")?
        .error_for_status()
        .context("The Storage Assistant model server rejected the download")?;
    let total = response.content_length().unwrap_or(MODEL_EXPECTED_BYTES);

    let result = (|| -> Result<(u64, String)> {
        let mut file = File::create(&partial)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; BUFFER_BYTES];
        let mut downloaded = 0_u64;
        let mut last_emitted = 0_u64;

        emit_progress(app, "downloading", downloaded, total);
        loop {
            let read = response
                .read(&mut buffer)
                .context("The Storage Assistant download ended unexpectedly")?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            hasher.update(&buffer[..read]);
            downloaded = downloaded.saturating_add(read as u64);
            if downloaded.saturating_sub(last_emitted) >= PROGRESS_INTERVAL_BYTES {
                emit_progress(app, "downloading", downloaded, total);
                last_emitted = downloaded;
            }
        }
        file.sync_all()?;
        Ok((downloaded, hex::encode(hasher.finalize())))
    })();

    let (downloaded, digest) = match result {
        Ok(result) => result,
        Err(error) => {
            let _ = fs::remove_file(&partial);
            return Err(error);
        }
    };

    emit_progress(app, "verifying", downloaded, total);
    if !digest.eq_ignore_ascii_case(MODEL_SHA256) {
        let _ = fs::remove_file(&partial);
        return Err(anyhow!(
            "Storage Assistant verification failed. Expected {MODEL_SHA256}, received {digest}"
        ));
    }

    fs::rename(&partial, &target).context("Unable to finalize the Storage Assistant model")?;
    let manifest = ModelManifest {
        base_model: BASE_MODEL.to_string(),
        quantization: QUANTIZATION.to_string(),
        sha256: MODEL_SHA256.to_string(),
        bytes: downloaded,
        source: MODEL_URL.to_string(),
        license: "Apache-2.0".to_string(),
        installed_at: Utc::now().to_rfc3339(),
    };
    fs::write(manifest_path(), serde_json::to_vec_pretty(&manifest)?)?;
    emit_progress(app, "ready", downloaded, downloaded);
    Ok(())
}

fn emit_progress(app: &AppHandle, phase: &str, downloaded_bytes: u64, total_bytes: u64) {
    let _ = app.emit(
        "assistant-download-progress",
        AssistantDownloadProgress {
            phase: phase.to_string(),
            downloaded_bytes,
            total_bytes,
        },
    );
}

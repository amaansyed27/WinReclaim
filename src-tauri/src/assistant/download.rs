use super::{
    manifest_path, model_path, model_root, runtime_path, runtime_root, ModelManifest, BASE_MODEL,
    MODEL_EXPECTED_BYTES, MODEL_SHA256, MODEL_URL, QUANTIZATION, RUNTIME_ASSET,
    RUNTIME_EXPECTED_BYTES, RUNTIME_RELEASE_API, RUNTIME_TAG,
};
use crate::domain::AssistantDownloadProgress;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

const BUFFER_BYTES: usize = 1024 * 1024;
const PROGRESS_INTERVAL_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

struct RuntimeInstall {
    sha256: String,
    bytes: u64,
}

pub fn install(app: &AppHandle) -> Result<()> {
    if super::status(false).verified {
        emit_progress(app, "ready", 0, 0);
        return Ok(());
    }

    let root = model_root();
    fs::create_dir_all(&root)?;
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(60 * 60))
        .user_agent("WinReclaim/1.1 storage-assistant")
        .build()?;

    let runtime = install_runtime(app, &client)?;
    let (model_bytes, model_digest) = install_model(app, &client)?;

    let manifest = ModelManifest {
        base_model: BASE_MODEL.to_string(),
        quantization: QUANTIZATION.to_string(),
        sha256: model_digest,
        bytes: model_bytes,
        source: MODEL_URL.to_string(),
        license: "Apache-2.0".to_string(),
        installed_at: Utc::now().to_rfc3339(),
        runtime_tag: RUNTIME_TAG.to_string(),
        runtime_asset: RUNTIME_ASSET.to_string(),
        runtime_sha256: runtime.sha256,
        runtime_bytes: runtime.bytes,
    };
    fs::write(manifest_path(), serde_json::to_vec_pretty(&manifest)?)?;
    emit_progress(app, "ready", model_bytes, model_bytes);
    Ok(())
}

fn install_runtime(app: &AppHandle, client: &Client) -> Result<RuntimeInstall> {
    if let Some(manifest) = super::read_manifest() {
        if runtime_path().is_file()
            && manifest.runtime_tag == RUNTIME_TAG
            && manifest.runtime_asset == RUNTIME_ASSET
            && manifest.runtime_sha256.len() == 64
            && manifest.runtime_bytes > 0
        {
            return Ok(RuntimeInstall {
                sha256: manifest.runtime_sha256,
                bytes: manifest.runtime_bytes,
            });
        }
    }

    emit_progress(app, "runtime-connecting", 0, RUNTIME_EXPECTED_BYTES);
    let release = client
        .get(RUNTIME_RELEASE_API)
        .send()
        .context("Unable to query the pinned llama.cpp release")?
        .error_for_status()
        .context("GitHub rejected the pinned llama.cpp release request")?
        .json::<GithubRelease>()
        .context("The pinned llama.cpp release metadata was malformed")?;
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == RUNTIME_ASSET)
        .ok_or_else(|| anyhow!("The pinned llama.cpp runtime asset is unavailable"))?;
    let expected_digest = asset
        .digest
        .as_deref()
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .filter(|digest| digest.len() == 64)
        .ok_or_else(|| anyhow!("GitHub did not provide a SHA-256 digest for the runtime asset"))?
        .to_ascii_lowercase();

    let root = model_root();
    let archive = root.join(format!("{RUNTIME_ASSET}.partial"));
    let staging = root.join("runtime.partial");
    let _ = fs::remove_file(&archive);
    let _ = fs::remove_dir_all(&staging);

    let (downloaded, digest) = download_verified(
        app,
        client,
        &asset.browser_download_url,
        &archive,
        &expected_digest,
        asset.size.max(RUNTIME_EXPECTED_BYTES),
        "runtime-downloading",
        "runtime-verifying",
    )?;

    emit_progress(app, "runtime-extracting", downloaded, downloaded);
    fs::create_dir_all(&staging)?;
    let extraction = extract_runtime(&archive, &staging);
    let _ = fs::remove_file(&archive);
    if let Err(error) = extraction {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    if !staging.join(super::RUNTIME_EXE).is_file() {
        let _ = fs::remove_dir_all(&staging);
        return Err(anyhow!(
            "The verified llama.cpp archive did not contain {}",
            super::RUNTIME_EXE
        ));
    }

    let final_root = runtime_root();
    if final_root.exists() {
        fs::remove_dir_all(&final_root)
            .context("Unable to replace the existing Storage Assistant runtime")?;
    }
    if let Some(parent) = final_root.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&staging, &final_root)
        .context("Unable to finalize the Storage Assistant runtime")?;

    Ok(RuntimeInstall {
        sha256: digest,
        bytes: downloaded,
    })
}

fn install_model(app: &AppHandle, client: &Client) -> Result<(u64, String)> {
    let target = model_path();
    if target.is_file() {
        emit_progress(app, "verifying", 0, MODEL_EXPECTED_BYTES);
        let (bytes, digest) = hash_file(&target)?;
        if digest.eq_ignore_ascii_case(MODEL_SHA256) {
            return Ok((bytes, digest));
        }
        fs::remove_file(&target).context("Unable to replace the unverified assistant model")?;
    }

    let root = model_root();
    let partial = root.join(format!("{}.partial", super::MODEL_FILE));
    let _ = fs::remove_file(&partial);
    emit_progress(app, "connecting", 0, MODEL_EXPECTED_BYTES);
    let result = download_verified(
        app,
        client,
        MODEL_URL,
        &partial,
        MODEL_SHA256,
        MODEL_EXPECTED_BYTES,
        "downloading",
        "verifying",
    );
    let (downloaded, digest) = match result {
        Ok(result) => result,
        Err(error) => {
            let _ = fs::remove_file(&partial);
            return Err(error);
        }
    };

    fs::rename(&partial, &target).context("Unable to finalize the Storage Assistant model")?;
    Ok((downloaded, digest))
}

#[allow(clippy::too_many_arguments)]
fn download_verified(
    app: &AppHandle,
    client: &Client,
    url: &str,
    partial: &Path,
    expected_digest: &str,
    expected_bytes: u64,
    download_phase: &str,
    verify_phase: &str,
) -> Result<(u64, String)> {
    let mut response = client
        .get(url)
        .send()
        .context("Unable to start the optional component download")?
        .error_for_status()
        .context("The optional component server rejected the download")?;
    let total = response.content_length().unwrap_or(expected_bytes);
    let mut file = File::create(partial)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    let mut downloaded = 0_u64;
    let mut last_emitted = 0_u64;

    emit_progress(app, download_phase, downloaded, total);
    loop {
        let read = response
            .read(&mut buffer)
            .context("The optional component download ended unexpectedly")?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        downloaded = downloaded.saturating_add(read as u64);
        if downloaded.saturating_sub(last_emitted) >= PROGRESS_INTERVAL_BYTES {
            emit_progress(app, download_phase, downloaded, total);
            last_emitted = downloaded;
        }
    }
    file.sync_all()?;

    emit_progress(app, verify_phase, downloaded, total);
    let digest = hex::encode(hasher.finalize());
    if !digest.eq_ignore_ascii_case(expected_digest) {
        return Err(anyhow!(
            "Optional component verification failed. Expected {expected_digest}, received {digest}"
        ));
    }
    Ok((downloaded, digest))
}

fn hash_file(path: &Path) -> Result<(u64, String)> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    let mut bytes = 0_u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes = bytes.saturating_add(read as u64);
    }
    Ok((bytes, hex::encode(hasher.finalize())))
}

fn extract_runtime(archive_path: &Path, destination: &Path) -> Result<()> {
    let archive_file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(archive_file)
        .context("Unable to open the verified llama.cpp runtime archive")?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .context("Unable to read the llama.cpp runtime archive")?;
        let relative = entry
            .enclosed_name()
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("The llama.cpp runtime archive contained an unsafe path"))?;
        let output = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&output)?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut target = File::create(&output)?;
        io::copy(&mut entry, &mut target)?;
        target.sync_all()?;
    }
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

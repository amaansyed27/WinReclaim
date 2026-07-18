use crate::domain::CleanupReceipt;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn persist_receipt(receipt: &CleanupReceipt) -> Result<PathBuf> {
    let root = receipt_root();
    fs::create_dir_all(&root)?;
    let path = root.join(format!("{}.json", receipt.id));
    fs::write(&path, serde_json::to_vec_pretty(receipt)?)?;
    Ok(path)
}

fn receipt_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("WinReclaim").join("receipts")
}

use crate::platform::windows::{is_reparse_point, recycle_bin_snapshot};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

#[derive(Debug, Default, Clone, Copy)]
pub struct SizeStats {
    pub bytes: u64,
    pub entries: u64,
    pub skipped: u64,
}

pub fn directory_size(path: &Path, cancel: &AtomicBool) -> SizeStats {
    walk_size(path, cancel, None, None)
}

pub fn eligible_temp_size(path: &Path, cancel: &AtomicBool, minimum_age: Duration) -> SizeStats {
    walk_size(path, cancel, Some(minimum_age), None)
}

pub fn recognised_dump_size(path: &Path, cancel: &AtomicBool) -> SizeStats {
    walk_size(path, cancel, None, Some(&["dmp", "mdmp", "wer"]))
}

pub fn prefetch_size(path: &Path, cancel: &AtomicBool) -> SizeStats {
    walk_size(path, cancel, None, Some(&["pf"]))
}

pub fn recycle_bin_size() -> SizeStats {
    recycle_bin_snapshot()
        .map(|(bytes, entries)| SizeStats {
            bytes,
            entries,
            skipped: 0,
        })
        .unwrap_or_default()
}

fn walk_size(
    root: &Path,
    cancel: &AtomicBool,
    minimum_age: Option<Duration>,
    allowed_extensions: Option<&[&str]>,
) -> SizeStats {
    if !root.exists() {
        return SizeStats::default();
    }

    let mut stats = SizeStats::default();
    let mut stack = vec![PathBuf::from(root)];
    let now = SystemTime::now();

    while let Some(path) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => {
                stats.skipped += 1;
                continue;
            }
        };
        if is_reparse_point(&metadata) {
            stats.skipped += 1;
            continue;
        }
        if metadata.is_file() {
            stats.entries += 1;
            if let Some(extensions) = allowed_extensions {
                let extension = path
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if !extensions.iter().any(|candidate| *candidate == extension) {
                    continue;
                }
            }
            if let Some(age) = minimum_age {
                let modified = match metadata.modified() {
                    Ok(value) => value,
                    Err(_) => {
                        stats.skipped += 1;
                        continue;
                    }
                };
                if now.duration_since(modified).unwrap_or_default() < age {
                    continue;
                }
            }
            stats.bytes = stats.bytes.saturating_add(metadata.len());
            continue;
        }
        if metadata.is_dir() {
            let entries = match fs::read_dir(&path) {
                Ok(entries) => entries,
                Err(_) => {
                    stats.skipped += 1;
                    continue;
                }
            };
            for entry in entries {
                match entry {
                    Ok(entry) => stack.push(entry.path()),
                    Err(_) => stats.skipped += 1,
                }
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sizes_fixture_directory() {
        let root = std::env::temp_dir().join(format!("winreclaim-size-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let mut file = fs::File::create(root.join("sample.bin")).unwrap();
        file.write_all(&[1_u8; 64]).unwrap();
        let cancel = AtomicBool::new(false);
        let result = directory_size(&root, &cancel);
        assert_eq!(result.bytes, 64);
        assert_eq!(result.entries, 1);
        fs::remove_dir_all(root).unwrap();
    }
}

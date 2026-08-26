use crate::db::repositories::sources::{ScanSnapshot, ScannedFile};
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

const MAX_DEPTH: usize = 32;
const MAX_FILES: usize = 50_000;
const EXCLUDED_DIRECTORIES: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".cache",
    "__pycache__",
    ".venv",
    "venv",
];

#[derive(Default)]
pub struct ContextRuntime {
    scanning: Mutex<HashSet<String>>,
}

impl ContextRuntime {
    pub fn begin(&self, source_id: &str) -> Result<(), String> {
        let mut scans = self
            .scanning
            .lock()
            .map_err(|_| "Context scan state is unavailable")?;
        if !scans.insert(source_id.to_string()) {
            return Err("This Source is already being scanned".into());
        }
        Ok(())
    }
    pub fn finish(&self, source_id: &str) {
        if let Ok(mut scans) = self.scanning.lock() {
            scans.remove(source_id);
        }
    }
}

fn path_text(path: &Path, label: &str) -> Result<String, String> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{label} contains unsupported Unicode"))
}

pub fn authorize_directory(selected: &Path, app_data: &Path) -> Result<PathBuf, String> {
    if !selected.is_absolute() {
        return Err("A Source must be an absolute directory".into());
    }
    let root = selected
        .canonicalize()
        .map_err(|e| format!("Selected Source is unavailable: {e}"))?;
    if !root.is_dir() {
        return Err("A Source must be a directory".into());
    }
    if root.parent().is_none() {
        return Err("A filesystem root cannot be indexed as a Source".into());
    }
    let app_data = app_data
        .canonicalize()
        .unwrap_or_else(|_| app_data.to_path_buf());
    if root.starts_with(&app_data) || app_data.starts_with(&root) {
        return Err("A Source cannot contain Aether's application data".into());
    }
    Ok(root)
}

#[cfg(windows)]
fn is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn is_reparse(_: &fs::Metadata) -> bool {
    false
}

fn millis(time: Result<std::time::SystemTime, std::io::Error>) -> Option<i64> {
    time.ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis()
        .try_into()
        .ok()
}

fn relative_text(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "Indexed file escaped its Source")?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .ok_or("Indexed path contains unsupported Unicode")?,
            ),
            _ => return Err("Indexed file has an unsafe relative path".into()),
        }
    }
    if parts.is_empty() {
        return Err("Indexed file has an empty relative path".into());
    }
    Ok(parts.join("/"))
}

pub fn scan_directory(root: &Path) -> Result<ScanSnapshot, String> {
    scan_with_limits(root, MAX_DEPTH, MAX_FILES)
}

fn scan_with_limits(
    root: &Path,
    max_depth: usize,
    max_files: usize,
) -> Result<ScanSnapshot, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("Source directory is unavailable: {e}"))?;
    if !root.is_dir() {
        return Err("Source directory is no longer available".into());
    }
    let mut stack = vec![(root.clone(), 0usize)];
    let mut files = Vec::new();
    let mut skipped = 0u32;
    let mut errors = 0u32;
    let mut truncated = false;
    while let Some((directory, depth)) = stack.pop() {
        if depth > max_depth {
            skipped += 1;
            truncated = true;
            continue;
        }
        let entries = match fs::read_dir(&directory) {
            Ok(v) => v,
            Err(_) => {
                errors += 1;
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            if metadata.file_type().is_symlink() || is_reparse(&metadata) {
                skipped += 1;
                continue;
            }
            if metadata.is_dir() {
                let excluded = entry.file_name().to_str().is_some_and(|name| {
                    EXCLUDED_DIRECTORIES
                        .iter()
                        .any(|item| name.eq_ignore_ascii_case(item))
                });
                if excluded {
                    skipped += 1;
                } else if depth == max_depth {
                    skipped += 1;
                    truncated = true;
                } else {
                    stack.push((path, depth + 1));
                }
                continue;
            }
            if !metadata.is_file() {
                skipped += 1;
                continue;
            }
            if files.len() >= max_files {
                truncated = true;
                break;
            }
            let relative_path = match relative_text(&root, &path) {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            let filename = match path.file_name().and_then(|v| v.to_str()) {
                Some(v) if !v.is_empty() => v.to_string(),
                _ => {
                    errors += 1;
                    continue;
                }
            };
            let extension = path
                .extension()
                .and_then(|v| v.to_str())
                .filter(|v| !v.is_empty())
                .map(str::to_ascii_lowercase);
            let size_bytes = match i64::try_from(metadata.len()) {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            files.push(ScannedFile {
                relative_path,
                filename,
                extension,
                size_bytes,
                created_at_fs: millis(metadata.created()),
                modified_at_fs: millis(metadata.modified()),
            });
        }
        if truncated && files.len() >= max_files {
            break;
        }
    }
    Ok(ScanSnapshot {
        files,
        skipped,
        errors,
        truncated,
    })
}

pub fn canonical_text(path: &Path) -> Result<String, String> {
    path_text(path, "Source path")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    #[test]
    fn validates_scope_and_scans_metadata_without_generated_directories() {
        let temp = TempDir::new().unwrap();
        let app = temp.path().join("app");
        let source = temp.path().join("source");
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(source.join("node_modules")).unwrap();
        fs::write(source.join("one.md"), "one").unwrap();
        fs::write(source.join("node_modules/skip.js"), "skip").unwrap();
        assert!(authorize_directory(&source, &app).is_ok());
        assert!(authorize_directory(&app, &app).is_err());
        assert!(authorize_directory(temp.path(), &app).is_err());
        assert!(authorize_directory(&source.join("one.md"), &app).is_err());
        let filesystem_root = temp.path().ancestors().last().unwrap();
        assert!(authorize_directory(filesystem_root, &app).is_err());
        let scan = scan_with_limits(&source, 4, 10).unwrap();
        assert_eq!(scan.files.len(), 1);
        assert_eq!(scan.files[0].relative_path, "one.md");
        assert_eq!(scan.skipped, 1);
    }
    #[test]
    fn truncation_is_explicit() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a"), "a").unwrap();
        fs::write(temp.path().join("b"), "b").unwrap();
        let scan = scan_with_limits(temp.path(), 2, 1).unwrap();
        assert_eq!(scan.files.len(), 1);
        assert!(scan.truncated);
    }
    #[cfg(unix)]
    #[test]
    fn does_not_follow_symlinks() {
        use std::os::unix::fs::symlink;
        let temp = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        fs::write(outside.path().join("secret"), "x").unwrap();
        symlink(outside.path(), temp.path().join("link")).unwrap();
        let scan = scan_directory(temp.path()).unwrap();
        assert!(scan.files.is_empty());
        assert_eq!(scan.skipped, 1);
    }
}

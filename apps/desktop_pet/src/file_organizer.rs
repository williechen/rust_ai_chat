use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileOrganizerError {
    #[error("desktop pet must not run as root")]
    RootProcess,

    #[error("authorized root does not exist or cannot be resolved: {0}")]
    InvalidRoot(String),

    #[error("authorized root is not a directory")]
    RootNotDirectory,

    #[error("filesystem scan failed: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FilePreview {
    pub id: String,
    pub relative_path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SizeCandidateGroup {
    pub size_bytes: u64,
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanWarning {
    pub relative_path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanPreview {
    pub files: Vec<FilePreview>,
    pub duplicate_size_candidates: Vec<SizeCandidateGroup>,
    pub warnings: Vec<ScanWarning>,
}

#[cfg(unix)]
fn ensure_non_root_process() -> Result<(), FileOrganizerError> {
    let euid = unsafe { libc::geteuid() };
    reject_root_euid(euid)
}

#[cfg(not(unix))]
fn ensure_non_root_process() -> Result<(), FileOrganizerError> {
    Ok(())
}

#[derive(Debug, Clone)]
pub struct AuthorizedRoot {
    canonical: PathBuf,
}

impl AuthorizedRoot {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, FileOrganizerError> {
        ensure_non_root_process()?;

        let canonical = fs::canonicalize(path.as_ref())
            .map_err(|error| FileOrganizerError::InvalidRoot(error.to_string()))?;

        if !canonical.is_dir() {
            return Err(FileOrganizerError::RootNotDirectory);
        }

        Ok(Self { canonical })
    }

    fn path(&self) -> &Path {
        &self.canonical
    }
}

pub fn scan_preview(root: &AuthorizedRoot) -> Result<ScanPreview, FileOrganizerError> {
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    scan_dir(root.path(), root.path(), &mut files, &mut warnings)?;

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    for (index, file) in files.iter_mut().enumerate() {
        file.id = format!("item-{index:06}");
    }

    let duplicate_size_candidates = build_size_candidates(&files);

    Ok(ScanPreview {
        files,
        duplicate_size_candidates,
        warnings,
    })
}

fn scan_dir(
    root: &Path,
    current: &Path,
    files: &mut Vec<FilePreview>,
    warnings: &mut Vec<ScanWarning>,
) -> Result<(), FileOrganizerError> {
    let entries = fs::read_dir(current)?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warnings.push(ScanWarning {
                    relative_path: ".".to_string(),
                    message: error.to_string(),
                });
                continue;
            }
        };

        let path = entry.path();

        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                warnings.push(ScanWarning {
                    relative_path: display_relative(root, &path),
                    message: error.to_string(),
                });
                continue;
            }
        };

        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            if let Err(error) = scan_dir(root, &path, files, warnings) {
                warnings.push(ScanWarning {
                    relative_path: display_relative(root, &path),
                    message: error.to_string(),
                });
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        files.push(FilePreview {
            id: String::new(),
            relative_path: display_relative(root, &path),
            size_bytes: metadata.len(),
        });
    }

    Ok(())
}

fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn build_size_candidates(files: &[FilePreview]) -> Vec<SizeCandidateGroup> {
    let mut by_size: BTreeMap<u64, Vec<String>> = BTreeMap::new();

    for file in files {
        by_size
            .entry(file.size_bytes)
            .or_default()
            .push(file.id.clone());
    }

    by_size
        .into_iter()
        .filter_map(|(size_bytes, item_ids)| {
            (item_ids.len() >= 2).then_some(SizeCandidateGroup {
                size_bytes,
                item_ids,
            })
        })
        .collect()
}

fn reject_root_euid(euid: u32) -> Result<(), FileOrganizerError> {
    if euid == 0 {
        Err(FileOrganizerError::RootProcess)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock before unix epoch")
                .as_nanos();

            let path = std::env::temp_dir()
                .join(format!("desktop-pet-{name}-{}-{nonce}", std::process::id()));

            fs::create_dir_all(&path).expect("create test dir");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn scans_regular_files_and_builds_size_candidates() {
        let sandbox = TestDir::new("scan");
        fs::write(sandbox.path.join("a.txt"), b"abc").unwrap();
        fs::write(sandbox.path.join("b.txt"), b"xyz").unwrap();
        fs::write(sandbox.path.join("c.txt"), b"longer").unwrap();

        let root = AuthorizedRoot::new(&sandbox.path).unwrap();
        let preview = scan_preview(&root).unwrap();

        assert_eq!(preview.files.len(), 3);
        assert_eq!(preview.duplicate_size_candidates.len(), 1);
        assert_eq!(preview.duplicate_size_candidates[0].size_bytes, 3);
        assert_eq!(preview.duplicate_size_candidates[0].item_ids.len(), 2);
    }

    #[test]
    fn ids_are_opaque_not_absolute_paths() {
        let sandbox = TestDir::new("opaque-id");
        fs::write(sandbox.path.join("secret.txt"), b"x").unwrap();

        let root = AuthorizedRoot::new(&sandbox.path).unwrap();
        let preview = scan_preview(&root).unwrap();

        let file = &preview.files[0];
        assert!(file.id.starts_with("item-"));
        assert!(
            !file
                .id
                .contains(&sandbox.path.to_string_lossy().to_string())
        );
        assert_eq!(file.relative_path, "secret.txt");
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlink_instead_of_following_it() {
        use std::os::unix::fs::symlink;

        let sandbox = TestDir::new("symlink");
        let outside = TestDir::new("outside");

        fs::write(outside.path.join("outside.txt"), b"secret").unwrap();
        symlink(&outside.path, sandbox.path.join("escape")).unwrap();

        let root = AuthorizedRoot::new(&sandbox.path).unwrap();
        let preview = scan_preview(&root).unwrap();

        assert!(preview.files.is_empty());
    }

    #[test]
    fn rejects_file_as_authorized_root() {
        let sandbox = TestDir::new("root-file");
        let file = sandbox.path.join("not-a-dir.txt");
        fs::write(&file, b"x").unwrap();

        let error = AuthorizedRoot::new(file).unwrap_err();
        assert!(matches!(error, FileOrganizerError::RootNotDirectory));
    }

    #[test]
    fn rejects_root_euid() {
        assert!(matches!(
            reject_root_euid(0),
            Err(FileOrganizerError::RootProcess)
        ));
        assert!(reject_root_euid(1000).is_ok());
    }
}

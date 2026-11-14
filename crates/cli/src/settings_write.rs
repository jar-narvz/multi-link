//! Atomic settings replacement, independent of environment and settings merging.
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Snapshot {
    pub path: PathBuf,
    pub original: Option<Vec<u8>>,
    pub mode: Option<u32>,
}
impl Snapshot {
    pub fn read(path: PathBuf) -> io::Result<Self> {
        let (original, mode) = match fs::metadata(&path) {
            Ok(meta) => {
                if !meta.is_file() {
                    return Err(io::Error::other("settings must be a regular file"));
                }
                (
                    Some(fs::read(&path)?),
                    Some(meta.permissions().mode() & 0o7777),
                )
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => (None, None),
            Err(e) => return Err(e),
        };
        Ok(Self {
            path,
            original,
            mode,
        })
    }
    pub fn text(&self) -> io::Result<&str> {
        std::str::from_utf8(self.original.as_deref().unwrap_or(b"{}"))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub fn backup_path(path: &Path) -> io::Result<PathBuf> {
    let mut random = [0u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut random)?;
    let random: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let mut name = path.as_os_str().to_os_string();
    name.push(format!(".multi-link-backup-{timestamp}-{random}"));
    Ok(name.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Write,
    Backup,
    BeforeRename,
    Rename,
}

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

/// Fault hooks let tests fail operations or edit the target at the race boundary.
/// There is still a race between the final comparison and rename; no portable
/// filesystem operation atomically compares file bytes and replaces the file.
pub fn replace(
    source: &Snapshot,
    bytes: &[u8],
    backup: &Path,
    mut fault: impl FnMut(Stage) -> io::Result<()>,
) -> io::Result<()> {
    if source.original.as_deref() == Some(bytes) {
        return Ok(());
    }
    let parent = source
        .path
        .parent()
        .ok_or_else(|| io::Error::other("settings has no parent"))?;
    fs::create_dir_all(parent)?;
    let mut temp_name = backup.as_os_str().to_os_string();
    temp_name.push(".tmp");
    let temp_path = PathBuf::from(temp_name);
    // Do not remove somebody else's file if create_new refuses it.
    let mut file = private_file(&temp_path)?;
    let temp = Temp(temp_path);
    fault(Stage::Write)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    if let Some(original) = &source.original {
        fault(Stage::Backup)?;
        let mut saved = private_file(backup)?;
        saved.write_all(original)?;
        saved.sync_all()?;
    }
    file.set_permissions(fs::Permissions::from_mode(source.mode.unwrap_or(0o600)))?;
    fault(Stage::BeforeRename)?;
    // Refuse non-regular replacements (including symlinks) without reading them.
    let current = match fs::symlink_metadata(&source.path) {
        Ok(meta) if meta.is_file() => Some(fs::read(&source.path)?),
        Ok(_) => return Err(io::Error::other("settings changed since read")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    if current != source.original {
        return Err(io::Error::other("settings changed since read"));
    }
    fault(Stage::Rename)?;
    fs::rename(&temp.0, &source.path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn staged_temp_and_backup_are_private_and_original_mode_is_restored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, b"original").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let source = Snapshot::read(path.clone()).unwrap();
        let backup = backup_path(&path).unwrap();
        let mut temp_name = backup.as_os_str().to_os_string();
        temp_name.push(".tmp");
        let temp = PathBuf::from(temp_name);
        replace(&source, b"new", &backup, |at| {
            if at == Stage::Write {
                assert_eq!(temp.parent(), path.parent());
                assert_eq!(fs::metadata(&temp)?.permissions().mode() & 0o777, 0o600);
            }
            if at == Stage::BeforeRename {
                assert_eq!(fs::read(&backup)?, b"original");
                assert_eq!(fs::metadata(&backup)?.permissions().mode() & 0o777, 0o600);
                assert_eq!(fs::read(&path)?, b"original");
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o644
        );
        assert!(!temp.exists());
    }

    #[test]
    fn faults_and_changed_bytes_preserve_target_and_clean_temp() {
        for stage in [
            Stage::Write,
            Stage::Backup,
            Stage::BeforeRename,
            Stage::Rename,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("settings.json");
            fs::write(&path, b"original").unwrap();
            let source = Snapshot::read(path.clone()).unwrap();
            let backup = backup_path(&path).unwrap();
            assert!(
                replace(&source, b"new", &backup, |at| if at == stage {
                    Err(io::Error::other("injected"))
                } else {
                    Ok(())
                })
                .is_err()
            );
            assert_eq!(fs::read(&path).unwrap(), b"original");
            assert!(
                !dir.path().read_dir().unwrap().any(|p| p
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|e| e == "tmp"))
            );
        }
        for exists in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("settings.json");
            if exists {
                fs::write(&path, b"original").unwrap();
            }
            let source = Snapshot::read(path.clone()).unwrap();
            let backup = backup_path(&path).unwrap();
            let result = replace(&source, b"new", &backup, |at| {
                if at == Stage::BeforeRename {
                    fs::write(&path, b"concurrent edit")?;
                }
                Ok(())
            });
            assert_eq!(
                result.unwrap_err().to_string(),
                "settings changed since read"
            );
            assert_eq!(fs::read(&path).unwrap(), b"concurrent edit");
            assert!(
                !dir.path().read_dir().unwrap().any(|p| p
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|e| e == "tmp"))
            );
        }
    }
}

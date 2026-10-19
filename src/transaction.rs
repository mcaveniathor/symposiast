use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Stage bundle edits on the same filesystem and restore the old tree if
/// publication fails. Readers should reopen the bundle between requests.
pub struct Transaction {
    original: PathBuf,
    stage: tempfile::TempDir,
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "bundle transactions do not follow symlinks: {}",
            entry.path().display()
        );
        if kind.is_dir() {
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination.join(entry.file_name()))?;
        } else {
            anyhow::bail!("unsupported bundle file: {}", entry.path().display());
        }
    }
    Ok(())
}

impl Transaction {
    pub fn new(root: &Path) -> Result<Self> {
        let original = root.canonicalize()?;
        let stage = tempfile::Builder::new()
            .prefix(".bundle-stage-")
            .tempdir_in(
                original
                    .parent()
                    .context("bundle needs a parent directory")?,
            )?;
        copy_tree(&original, stage.path())?;
        Ok(Self { original, stage })
    }
    pub fn path(&self) -> &Path {
        self.stage.path()
    }
    pub fn preserve_failure(self) -> PathBuf {
        self.stage.keep()
    }

    pub fn commit(self, finalize: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
        let backup = tempfile::Builder::new()
            .prefix(".bundle-backup-")
            .tempdir_in(self.original.parent().unwrap())?;
        let previous = backup.path().join("previous");
        fs::rename(&self.original, &previous)?;
        if let Err(error) = fs::rename(self.stage.path(), &self.original) {
            fs::rename(&previous, &self.original)
                .context("restore original bundle after failed publication")?;
            return Err(error.into());
        }
        if let Err(error) = finalize(&self.original) {
            fs::rename(&self.original, self.stage.path()).context("retain failed publication")?;
            fs::rename(&previous, &self.original).context("restore original bundle")?;
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_finalize_restores_entire_previous_bundle() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("bundle");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("bundle.json"), "old").unwrap();
        let transaction = Transaction::new(&root).unwrap();
        fs::write(transaction.path().join("bundle.json"), "new").unwrap();
        assert!(
            transaction
                .commit(|_| anyhow::bail!("render failed"))
                .is_err()
        );
        assert_eq!(fs::read_to_string(root.join("bundle.json")).unwrap(), "old");
    }
    #[test]
    fn successful_commit_keeps_other_user_files() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("bundle");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("notes.md"), "notes").unwrap();
        let transaction = Transaction::new(&root).unwrap();
        fs::write(transaction.path().join("bundle.json"), "new").unwrap();
        transaction.commit(|_| Ok(())).unwrap();
        assert_eq!(fs::read_to_string(root.join("notes.md")).unwrap(), "notes");
        assert_eq!(fs::read_to_string(root.join("bundle.json")).unwrap(), "new");
    }
}

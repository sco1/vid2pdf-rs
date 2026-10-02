use anyhow::{Context, Result};
use std::path::Path;
use tempfile::{Builder, TempDir};

const PREFIX: &str = "vid2pdf-";

/// Return the directory containing `file`, guarding for files in the current directory (`.`)
fn parent_dir(file: &Path) -> &Path {
    match file.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    }
}

/// Utility wrapper for `tempfile::TempDir`
///
/// Use [NeighborTempDir::create_beside] to build a temporary directory in the parent directory of a
/// specified file.
///
/// While the directory and its contents should be removed when this is dropped, one can use
/// [`NeighborTempDir::close`] to remove the directory & provide feedback if an error occurs.
#[derive(Debug)]
pub(crate) struct NeighborTempDir(TempDir);

impl NeighborTempDir {
    /// Create a temporary directory next to `file`.
    pub(crate) fn create_beside(file: &Path) -> Result<Self> {
        let parent = parent_dir(file);
        let dir = Builder::new()
            .prefix(PREFIX)
            .tempdir_in(parent)
            .with_context(|| {
                format!(
                    "Failed to create temporary directory in '{}'",
                    parent.display()
                )
            })?;

        Ok(Self(dir))
    }

    pub(crate) fn path(&self) -> &Path {
        self.0.path()
    }

    /// Delete the temporary directory, reporting an error if encountered.
    pub(crate) fn close(self) -> Result<()> {
        let path = self.path().to_path_buf();
        self.0
            .close()
            .with_context(|| format!("Failed to remove temporary directory '{}'", path.display()))
    }
}

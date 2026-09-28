use anyhow::{Result, bail};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Returns true if `ffmpeg_exe` can be executed and `ffmpeg_exe --version` exits successfully
fn runs(ffmpeg_exe: &Path) -> bool {
    Command::new(ffmpeg_exe)
        .arg("-hide_banner")
        .arg("-version")
        .output()
        .is_ok_and(|out| out.status.success())
}

const FFMPEG_BIN: &str = if cfg!(windows) {
    "ffmpeg.exe"
} else {
    "ffmpeg"
};

/// Attempt to resolve a functioning ffmpeg executable.
///
/// Precedence is given as follows:
///   1. `base_dir`, if provided by the user (e.g. from the CLI or GUI)
///   2. `FFMPEG_PATH` environment variable, if set
///   3. `ffmpeg` on the system `PATH`
///
/// NOTE: `base_dir` and `FFMPEG_PATH` are assumed to be specified as ffmpeg's base directory, where
/// the binary is located at `<path>/bin/ffmpeg(.exe)`
pub fn resolve_ffmpeg(base_dir: Option<&Path>) -> Result<PathBuf> {
    let configured_path = base_dir.map(|dir| ("USER", dir.to_path_buf())).or_else(|| {
        env::var_os("FFMPEG_PATH")
            .filter(|v| !v.is_empty())
            .map(|v| ("ENV", PathBuf::from(v)))
    });

    match configured_path {
        // Try user spec or env var
        Some((origin, base)) => {
            let ffmpeg_bin = base.join("bin").join(FFMPEG_BIN);
            if runs(&ffmpeg_bin) {
                Ok(ffmpeg_bin)
            } else {
                bail!(
                    "could not run ffmpeg at '{}' (from {origin})",
                    ffmpeg_bin.display()
                )
            }
        }
        // Fallthrough to system PATH
        None => {
            let ffmpeg_bin = PathBuf::from(FFMPEG_BIN);
            if runs(&ffmpeg_bin) {
                Ok(ffmpeg_bin)
            } else {
                bail!(
                    "Could not implicitly resolve ffmpeg; ffmpeg was not found on the system PATH \
                    and no FFMPEG_PATH env var was found"
                )
            }
        }
    }
}

/// Image format used for the intermediate frames.
///
/// Available options:
///   * PNG - Retain video resolution, typically higher run time & resulting PDF file size
///   * JPEG - Lossy video frame compression, typically lower run time & resulting PDF file size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameFormat {
    #[default]
    Png,
    Jpeg,
}

impl FrameFormat {
    /// Map to extension expected by ffmpeg.
    pub fn extension(self) -> &'static str {
        match self {
            FrameFormat::Png => "png",
            FrameFormat::Jpeg => "jpg",
        }
    }
}

use crate::temp_dir::NeighborTempDir;
use anyhow::{Context, Result, bail, ensure};
use std::env;
use std::fs;
use std::io;
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
#[derive(Debug, Clone, Copy)]
pub enum FrameFormat {
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

#[derive(Debug)]
pub struct ExtractOptions {
    pub start: Option<String>,
    pub end: Option<String>,
    pub format: FrameFormat,
}

#[derive(Debug)]
pub struct Frames {
    dir: NeighborTempDir,
    paths: Vec<PathBuf>,
}

impl Frames {
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn close(self) -> Result<()> {
        self.dir.close()
    }
}

pub fn extract_frames(ffmpeg_exe: &Path, source: &Path, opts: &ExtractOptions) -> Result<Frames> {
    ensure!(
        source.is_file(),
        "Source video does not exist: '{}'",
        source.display()
    );

    let tmpdir = NeighborTempDir::create_beside(source)?;

    let mut cmd = Command::new(ffmpeg_exe);
    cmd.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
    if let Some(start) = &opts.start {
        cmd.args(["-ss", start]);
    }
    if let Some(end) = &opts.end {
        cmd.args(["-to", end]);
    }

    let pattern = format!("frame_%05d.{}", opts.format.extension());
    cmd.arg("-i").arg(source).arg(tmpdir.path().join(pattern));

    let output = cmd
        .output()
        .with_context(|| format!("failed to launch ffmpeg at '{}'", ffmpeg_exe.display()))?;
    if !output.status.success() {
        bail!(
            "ffmpeg exited with {}:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let mut paths = fs::read_dir(tmpdir.path())?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()
        .context("Failed to list extracted frames")?;
    paths.sort();

    ensure!(
        !paths.is_empty(),
        "ffmpeg did not extract any frames, please check your start/end parameters"
    );

    Ok(Frames { dir: tmpdir, paths })
}

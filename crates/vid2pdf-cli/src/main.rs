use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use vid2pdf_core::{ExtractOptions, FrameFormat, extract_frames, resolve_ffmpeg};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliFrameFormat {
    Png,
    Jpeg,
}

impl From<CliFrameFormat> for FrameFormat {
    fn from(f: CliFrameFormat) -> Self {
        match f {
            CliFrameFormat::Png => FrameFormat::Png,
            CliFrameFormat::Jpeg => FrameFormat::Jpeg,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "vid2pdf",
    version,
    about = "Convert a video file to a PDF image series."
)]
struct Cli {
    /// Source video
    source: PathBuf,

    /// Override destination directory
    #[arg(short, long, value_name = "DIRECTORY")]
    out_dir: Option<PathBuf>,

    /// Start time, as [HH:]MM:SS[.m...] or S+[.m...] (omit for video start)
    #[arg(short, long)]
    start: Option<String>,

    /// End time, as [HH:]MM:SS[.m...] or S+[.m...] (omit for video end)
    #[arg(short, long)]
    end: Option<String>,

    /// Intermediate frame format
    #[arg(short = 'f', long, value_enum, default_value_t = CliFrameFormat::Png)]
    frame_type: CliFrameFormat,

    /// ffmpeg directory override
    #[arg(long, value_name = "PATH")]
    ffmpeg_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok(); // Preserve any exisiting env var
    let cli = Cli::parse();

    let ffmpeg = resolve_ffmpeg(cli.ffmpeg_path.as_deref())?;
    let opts = ExtractOptions {
        start: cli.start,
        end: cli.end,
        format: cli.frame_type.into(),
    };

    let frames = extract_frames(&ffmpeg, &cli.source, &opts)?;
    println!("Extracted {} frames", frames.paths().len());

    // TODO: PDF Generation
    frames.close()
}

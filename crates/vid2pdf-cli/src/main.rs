use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use vid2pdf_core::FrameFormat;

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
    start: Option<String>, // TODO

    /// End time, as [HH:]MM:SS[.m...] or S+[.m...] (omit for video end)
    #[arg(short, long)]
    end: Option<String>, // TODO

    /// Intermediate frame format
    #[arg(short = 'f', long, value_enum, default_value_t = CliFrameFormat::Png)]
    frame_type: CliFrameFormat,

    /// ffmpeg directory override
    #[arg(long, value_name = "PATH")]
    ffmpeg_path: Option<PathBuf>,
}

fn main() {
    dotenvy::dotenv().ok(); // Preserve any exisiting env var
    let _ = Cli::parse();
}

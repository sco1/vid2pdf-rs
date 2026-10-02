pub mod ffmpeg;
mod temp_dir;

pub use ffmpeg::{ExtractOptions, FrameFormat, extract_frames, resolve_ffmpeg};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

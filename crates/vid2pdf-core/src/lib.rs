pub mod ffmpeg;

pub use ffmpeg::{FrameFormat, resolve_ffmpeg};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

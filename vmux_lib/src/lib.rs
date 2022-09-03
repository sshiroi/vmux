mod chapters;
pub use crate::chapters::*;

mod av_source;
pub use crate::av_source::*;

//pub use bluray_support::BD;
pub use bluray_support::*;

//pub use ffms2;

mod cmd_line_args;
pub use cmd_line_args::*;

mod vmapping;
pub use vmapping::*;

pub mod matroska;

pub mod bd_cache;
pub mod bd_stream_av_cache;

pub mod handling {
    //TODO: remove all references to "handling"
    pub use crate::config::*;
}

pub use ffms2;

pub mod frame_cache;
pub mod fs;
pub mod matroska_backed;
pub mod matroska_hellofs;
pub mod mpv_script;
pub mod wavy;
pub mod y4m_video_helper;
pub mod y4m_wav_backed_file;
pub mod y4my;

pub mod config;

pub use serde_json;
//pub use rmp_serde;

#[derive(Copy, Clone)]
pub struct ClipRecut {
    pub offset: u64,
    pub duration: Option<u64>,
}

impl ClipRecut {
    pub fn offset_duration(offset: u64, duration: u64) -> Self {
        Self {
            offset,
            duration: Some(duration),
        }
    }
    pub fn offset_end(offset: u64) -> Self {
        Self {
            offset,
            duration: None,
        }
    }
}
impl Default for ClipRecut {
    fn default() -> Self {
        Self {
            offset: 0,
            duration: None,
        }
    }
}

//TODO: move format duration here
pub fn align_up(a: u64, align: u64) -> u64 {
    let remainder = a % align;
    if remainder == 0 {
        a
    } else {
        a + (align - remainder)
    }
}

pub fn init_ffms2() {
    ffms2::FFMS2::Init()
}
pub fn deint_ffms2() {
    println!("DRopped ffms2");
    let asd = ffms2::FFMS2 {};
    drop(asd);
}

pub fn format_duration(d: u64) -> String {
    let ms = d / (90_000 / 1000);
    let h = ms / (1000 * 60 * 60);
    let min = ms / (1000 * 60) - h * 60;
    let s = ms / (1000) - h * 60 - min * 60;

    format!("{:02}:{:02}:{:02}", h, min, s)
}

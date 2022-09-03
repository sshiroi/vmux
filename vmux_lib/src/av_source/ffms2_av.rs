use ffms2::{
    audio::AudioSource,
    frame::{Frame, FrameResolution},
    track::{Track, TrackType},
    video::{SeekMode, VideoProperties, VideoSource},
};
use std::sync::{Arc, Mutex};

use super::ffms2_index::*;

pub type SharedFFMSAVSource = Arc<Mutex<FFMS2AVSource>>;

pub struct FFMS2AVSource {
    pub vs: VideoSource,
    pub vp: VideoProperties,

    pub audio: Vec<AudioSource>,

    pub ffms2_video_track: usize,

    //    pub frame0: Frame,
    pub frame0_rslu: FrameResolution,

    pub framerate_n: u64,
    pub framerate_d: u64,

    pub sar_n: u64,
    pub sar_d: u64,

    pub num_frames: u64,

    pub first_pts: u64,
}

impl FFMS2AVSource {
    pub fn new(idx: &FFMS2IndexedFile) -> Option<FFMS2AVSource> {
        let mut video = None;
        let mut audio_idxs = vec![];

        for x in 0..idx.index.NumTracks() {
            let t = idx.indexer.TrackTypeI(x);

            match t {
                TrackType::TYPE_VIDEO => video = Some(x),
                TrackType::TYPE_AUDIO => audio_idxs.push(x),
                _ => {}
            }
        }

        if video.is_none() {
            return None;
        }
        let video = video.unwrap();

        let mut vsa =
            VideoSource::new(&idx.file_path, video, &idx.index, 1, SeekMode::SEEK_NORMAL).unwrap();
        let vpa = vsa.GetVideoProperties();

        let num_framesa = vpa.NumFrames as u64;

        let mut audio = Vec::new();
        for a in &audio_idxs {
            match AudioSource::new(&idx.file_path, *a, &idx.index, video as _) {
                Ok(e) => audio.push(e),
                Err(_) => {} //  Err(e) => println!("{:?}",e),
            }
        }

        let frame0 = Frame::GetFrame(&mut vsa, 0).unwrap();
        let t = Track::TrackFromVideo(&mut vsa);

        let first_pts = t.FrameInfo(0).PTS as u64;

        //        println!("a");
        //        for x in 0..num_framesa {
        //            //     let fi = t.FrameInfo(x as _);
        //        }
        //        println!("b");

        println!("RFFDenominator: {}", vpa.RFFDenominator);
        println!("RFFNumerator: {}", vpa.RFFNumerator);

        println!("ColorSpace: {}", frame0.ColorSpace);
        println!("ColorRange: {}", frame0.ColorRange);
        println!("ColorPrima: {}", frame0.ColorPrimaries);
        println!("ConvertedPixelFormat: {}", frame0.ConvertedPixelFormat);
        println!("EncodedPixelFormat: {}", frame0.EncodedPixelFormat);

        println!("ScaledWidth: {}", frame0.ScaledWidth);
        println!("ScaledHeight: {}", frame0.ScaledHeight);
        println!("EncodedWidth: {}", frame0.EncodedWidth);
        println!("EncodedHeight: {}", frame0.EncodedHeight);
        for x in 0..4 {
            println!("line{}: {}", x, frame0.Linesize[x]);
        }
        println!("InterlacedFrame: {}", frame0.InterlacedFrame);
        println!("TopFieldFirst: {}", frame0.TopFieldFirst);

        Some(FFMS2AVSource {
            frame0_rslu: frame0.get_frame_resolution(),
            first_pts,
            ffms2_video_track: video,
            framerate_n: vpa.FPSNumerator as u64,
            framerate_d: vpa.FPSDenominator as u64,
            sar_n: vpa.SARNum as u64,
            sar_d: vpa.SARDen as u64,
            num_frames: num_framesa,
            vs: vsa,
            vp: vpa,
            audio,
        })
    }
}

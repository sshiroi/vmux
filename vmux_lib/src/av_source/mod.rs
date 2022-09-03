use std::path::*;
use std::sync::{Arc, Mutex};

mod ffms2_index;
pub use ffms2_index::*;

mod asource;
pub use asource::*;

mod vsource;
pub use vsource::*;

mod ffms2_av;
pub use ffms2_av::*;

type FrameData = Vec<u8>;

#[derive(Clone)]
pub struct AVSource {
    pub shared: SharedFFMSAVSource,
}

impl AVSource {
    pub fn from_ffms2<A: AsRef<Path>, B: AsRef<Path>>(file_path: A, index_path: B) -> AVSource {
        let idx = FFMS2IndexedFile::new(file_path, index_path);
        let shared = FFMS2AVSource::new(&idx).unwrap();

        AVSource {
            shared: Arc::new(Mutex::new(shared)),
        }
    }

    pub fn video(&self) -> VSource {
        let shr = self.shared.lock().unwrap();

        let rs = &shr.frame0_rslu;

        VSource {
            fp: VSourceFrameProvider::FFMS2(self.shared.clone()),

            //TODO: use frameinfo on a per frame basis, instead of guessing based on framerate
            frame_time: 1.0 / (shr.framerate_n as f64 / shr.framerate_d as f64),

            framerate_n: shr.framerate_n,
            framerate_d: shr.framerate_d,
            sar_d: shr.sar_d,
            sar_n: shr.sar_n,
            num_frames: shr.num_frames,

            //TODO: EncodedWidth ??
            width: rs.width as _,
            height: rs.height as _,
        }
    }

    pub fn audios(&self) -> Vec<ASource> {
        let shr = self.shared.lock().unwrap();

        let mut asd = Vec::new();
        for (i, a) in shr.audio.iter().enumerate() {
            let ap = a.GetAudioProperties();

            //TODO: dont harcode
            let frmt = match ap.SampleFormat {
                0 => AFormat::INT,
                1 => AFormat::INT,
                2 => AFormat::INT,
                3 => AFormat::FLOAT,
                _ => panic!("invalid format"),
            };

            asd.push(ASource {
                fp: ASourceFrameProvider::FFMS2(self.shared.clone()),
                num_samples: ap.NumSamples as _,
                audio_idx: i,
                channels: ap.Channels as _,
                bits_per_sample: ap.BitsPerSample as _,
                sample_rate: ap.SampleRate as _,
                format: frmt,
            })
        }
        asd
    }
}

use super::ffms2_av::*;

#[derive(Clone)]
pub enum AFormat {
    FLOAT,
    INT,
}

#[derive(Clone)]
pub enum ASourceFrameProvider {
    FFMS2(SharedFFMSAVSource),
    OtherCut((Box<ASource>, u64)),
    List(Vec<Box<ASource>>),
}

#[derive(Clone)]
pub struct ASource {
    pub(crate) fp: ASourceFrameProvider,
    pub(crate) audio_idx: usize,

    pub bits_per_sample: u64,
    pub sample_rate: u64,
    pub num_samples: u64,

    pub channels: u64,

    pub format: AFormat,
}

impl ASource {
    pub fn samples(&mut self, start: u64, cnt: u64) -> Vec<u8> {
        match &mut self.fp {
            ASourceFrameProvider::FFMS2(shared) => {
                let shr = shared.lock().unwrap();
                let asds: Vec<u8> = ffms2::audio::AudioSource::GetAudio(
                    &shr.audio[self.audio_idx],
                    start as _,
                    cnt as usize,
                )
                .unwrap();
                asds
            }
            ASourceFrameProvider::OtherCut((inner, astart)) => {
                inner.samples(*astart as u64 + start, cnt)
            }
            ASourceFrameProvider::List(others) => {
                let mut start = start;
                let mut cnt = cnt;

                let mut prev = Vec::new();

                let mut current_i = 0u64;
                for f in others {
                    if (start as u64) < current_i + f.num_samples {
                        let offset = start - current_i;
                        let max_request = f.num_samples - offset;
                        if max_request >= cnt {
                            prev.append(&mut f.samples(offset, cnt));
                            return prev;
                        } else {
                            prev.append(&mut f.samples(offset, max_request));
                            start += max_request;
                            cnt -= max_request;
                        }
                    }
                    current_i += f.num_samples;
                }
                panic!("ASD");
            }
        }
    }
    pub fn cut(&self, start: u64, count: u64) -> ASource {
        let mut new = self.clone();
        new.fp = ASourceFrameProvider::OtherCut((Box::new(self.clone()), start));
        new.num_samples = u64::min(count, new.num_samples - start);
        new
    }
    pub fn list(&mut self, other: Vec<ASource>) -> ASource {
        let mut base = self.clone();

        base.num_samples = 0;
        for f in &other {
            base.num_samples += f.num_samples;
        }
        let other = other.into_iter().map(|e| Box::new(e)).collect();
        base.fp = ASourceFrameProvider::List(other);
        base
    }
}

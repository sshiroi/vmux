use std::io::{Cursor, Read};

use super::{ffms2_av::SharedFFMSAVSource, FrameData};
use ffms2::frame::Frame;

#[derive(Clone)]
pub enum VSourceFrameProvider {
    FFMS2(SharedFFMSAVSource),
    OtherCut((Box<VSource>, u64)),
    List(Vec<Box<VSource>>),
}

#[derive(Clone)]
pub struct VSource {
    pub(crate) fp: VSourceFrameProvider,

    pub frame_time: f64,

    pub framerate_n: u64,
    pub framerate_d: u64,
    pub sar_n: u64,
    pub sar_d: u64,
    pub num_frames: u64,

    pub width: u64,
    pub height: u64,
}

impl VSource {
    pub fn frame(&mut self, idx: usize) -> FrameData {
        match &mut self.fp {
            VSourceFrameProvider::FFMS2(shared) => {
                let mut shr = shared.lock().unwrap();

                let mut vid_frame2 = Vec::new();
                let frame = Frame::GetFrame(&mut shr.vs, idx).unwrap();

                let line_fixing = frame.EncodedWidth != frame.Linesize[0];

                let pix = frame.get_pixel_data();

                let mut i = 0;
                for f in pix {
                    if let Some(e) = f {
                        let mut tmp = Vec::new();

                        let e = if line_fixing {
                            let mut cur = if i == 0 {
                                Cursor::new(e)
                            } else {
                                Cursor::new(&e[0..e.len() / 2])
                            };
                            let real_linesize = if i == 0 {
                                frame.EncodedWidth as usize
                            } else {
                                frame.EncodedWidth as usize / 2
                            };

                            tmp.resize(real_linesize * frame.EncodedHeight as usize, 0);

                            for x in 0..frame.EncodedHeight as usize - 1 {
                                let oo = (x * real_linesize) as usize;
                                cur.read(&mut tmp[oo..oo + real_linesize as usize]).unwrap();
                                let mut skep = Vec::new();
                                skep.resize(
                                    (frame.Linesize[i] as usize - real_linesize) as usize,
                                    0,
                                );
                                cur.read(&mut skep).unwrap();
                            }
                            &tmp
                        } else {
                            e
                        };
                        //TODO: only for yuv420
                        if i == 0 {
                            vid_frame2.append(&mut e.to_owned());
                        } else {
                            vid_frame2.append(&mut e[0..e.len() / 2].to_owned());
                        }
                    }
                    i += 1;
                }
                assert!(i >= 1);

                vid_frame2
            }
            VSourceFrameProvider::OtherCut((inner, start)) => inner.frame(*start as usize + idx),
            VSourceFrameProvider::List(others) => {
                let mut current_i = 0u64;
                for f in others {
                    if (idx as u64) < current_i + f.num_frames {
                        return f.frame(idx - current_i as usize);
                    }
                    current_i += f.num_frames;
                }
                panic!("ASD");
            }
        }
    }
    pub fn cut(&mut self, start: u64, count: u64) -> VSource {
        let mut new = self.clone();
        new.fp = VSourceFrameProvider::OtherCut((Box::new(self.clone()), start));
        new.num_frames = u64::min(count, new.num_frames - start - 1);
        new
    }
    pub fn list(&mut self, other: Vec<VSource>) -> VSource {
        let mut base = self.clone();

        base.num_frames = 0;
        for f in &other {
            base.num_frames += f.num_frames;
        }
        let other = other.into_iter().map(|e| Box::new(e)).collect();
        base.fp = VSourceFrameProvider::List(other);
        base
    }
}

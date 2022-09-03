mod bindings;
use serde::*;
use std::ffi::*;

pub struct BD {
    bluray: *mut bindings::bluray,

    titit: u32,
}

unsafe impl Send for BD {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Clip {
    pub pkt_count: u32,

    pub start_time: u64, /* start media time, 90kHz, ("playlist time") */
    pub in_time: u64,    /* start timestamp, 90kHz */
    pub out_time: u64,   /* end timestamp, 90kHz */

    pub clip_id: [char; 6],
}

impl Clip {
    pub fn clip_id_as_str(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.clip_id[0], self.clip_id[1], self.clip_id[2], self.clip_id[3], self.clip_id[4],
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Chapter {
    pub idx: u32,
    pub start: u64,    /* start media time, 90kHz, ("playlist time") */
    pub duration: u64, /* duration */
    pub offset: u64,   /* distance from title start, bytes */
    pub clip_ref: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TitleInfo {
    pub idx: u32,
    pub playlist: u32,
    pub duration: u64,
    pub clip_count: u32,
    pub angle_count: u8,
    pub chapter_count: u32,
    pub mark_count: u32,
    pub clips: Vec<Clip>,
    pub chapters: Vec<Chapter>,
    pub marks: Vec<PlayMark>,
    //  pub clips: *mut BLURAY_CLIP_INFO,
    //  pub chapters: *mut BLURAY_TITLE_CHAPTER,
    //  pub marks: *mut BLURAY_TITLE_MARK,
    //pub mvc_base_view_r_flag: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PlayMarkType {
    #[default]
    Entry,
    Link,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayMark {
    pub idx: u32,
    pub pm_type: PlayMarkType,
    pub start: u64,    /* mark media time, 90kHz, ("playlist time") */
    pub duration: u64, /* time to next mark */
    //offset: u64,    /* mark distance from title start, bytes */
    pub clip_ref: u32,
}

impl BD {
    pub fn open(bd_path: &str) -> Option<BD> {
        let aa = CString::new(bd_path).unwrap();
        let bluray = unsafe { bindings::bd_open(aa.as_ptr(), 0 as _) };
        if bluray == std::ptr::null_mut() {
            return None;
        }
        let mut bdbd = BD { bluray, titit: 0 };
        bdbd.titit = bdbd.get_titles();
        Some(bdbd)
    }
    pub fn get_titles(&self) -> u32 {
        unsafe { bindings::bd_get_titles(self.bluray, 0, 0) }
    }
    pub fn chapter_pos(&self, chapter: u32) -> u64 {
        unsafe {
            let ads = bindings::bd_chapter_pos(self.bluray, chapter);
            assert!(ads >= 0);
            ads as u64
        }
    }
    pub fn get_title_info(&self, title_idx: u32, angle: u32) -> Option<TitleInfo> {
        unsafe {
            let ti = bindings::bd_get_title_info(self.bluray, title_idx, angle);

            if ti != 0 as _ {
                let chapts = {
                    let mut chapts = Vec::new();
                    for x in 0..(*ti).chapter_count {
                        let chpt = (*ti).chapters.offset(x as _);
                        chapts.push(Chapter {
                            idx: (*chpt).idx,
                            start: (*chpt).start,
                            duration: (*chpt).duration,
                            offset: (*chpt).offset,
                            clip_ref: (*chpt).clip_ref,
                        });
                    }
                    chapts
                };
                let clps = {
                    let mut clps = Vec::new();
                    for x in 0..(*ti).clip_count {
                        let clp = (*ti).clips.offset(x as _);
                        clps.push(Clip {
                            pkt_count: (*clp).pkt_count,
                            start_time: (*clp).start_time,
                            in_time: (*clp).in_time,
                            out_time: (*clp).out_time,
                            clip_id: [
                                ((*clp).clip_id[0] as u8) as char,
                                ((*clp).clip_id[1] as u8) as char,
                                ((*clp).clip_id[2] as u8) as char,
                                ((*clp).clip_id[3] as u8) as char,
                                ((*clp).clip_id[4] as u8) as char,
                                ((*clp).clip_id[5] as u8) as char,
                            ],
                        });
                    }
                    clps
                };

                let mrks = {
                    let mut marks = Vec::new();
                    for x in 0..(*ti).mark_count {
                        let mrk = (*ti).marks.offset(x as _);
                        marks.push(PlayMark {
                            idx: (*mrk).idx,
                            pm_type: match (*mrk).type_ {
                                1 => PlayMarkType::Entry,
                                2 => PlayMarkType::Link,
                                _ => unreachable!(),
                            },
                            start: (*mrk).start,
                            duration: (*mrk).duration,
                            clip_ref: (*mrk).clip_ref,
                        });
                    }
                    marks
                };
                let ret = Some(TitleInfo {
                    idx: (*ti).idx,
                    playlist: (*ti).playlist,

                    clip_count: (*ti).clip_count,
                    angle_count: (*ti).angle_count,
                    mark_count: (*ti).mark_count,

                    clips: clps,
                    chapters: chapts,
                    marks: mrks,

                    //                    duration: std::time::Duration::from_secs_f64(((*ti).duration) as f64 / 90000.0),
                    duration: ((*ti).duration),

                    chapter_count: (*ti).chapter_count,
                });
                bindings::bd_free_title_info(ti);
                ret
            } else {
                None
            }
        }
    }
    pub fn select_title(&mut self, title: u32) -> Result<(), ()> {
        unsafe {
            let asd = bindings::bd_select_title(self.bluray, title);

            if asd == 0 {
                Err(())
            } else if asd == 1 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub fn get_title_size(&mut self) -> u64 {
        unsafe { bindings::bd_get_title_size(self.bluray) }
    }

    pub fn seek(&mut self, to: u64) -> u64 {
        unsafe {
            let ret = bindings::bd_seek(self.bluray, to);
            assert!(ret >= 0);
            ret as _
        }
    }
    pub fn tell(&mut self) -> u64 {
        unsafe { bindings::bd_tell(self.bluray) }
    }
    pub fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        unsafe {
            let red = bindings::bd_read(self.bluray, buffer.as_mut_ptr(), buffer.len() as _);
            assert!(red >= 0);
            Some(red as _)
        }
    }
}

impl Drop for BD {
    fn drop(&mut self) {
        // println!("Dropped a BD");
        unsafe { bindings::bd_close(self.bluray) }
    }
}

use bluray_support::*;

use crate::ClipRecut;

pub fn create_chapter(start: u64, end: u64, title: &str) -> String {
    format!(
        r##"
[CHAPTER]
TIMEBASE=1/1000
START={}
END={}
title={}"##,
        start, end, title
    )
}

pub fn gen_chaps_for_title(bd: &BD, title_idx: u32, cr: ClipRecut) -> String {
    let ti = bd.get_title_info(title_idx, 0).unwrap();
    gen_chaps_for_title_ti(&ti, cr, true).0
}

pub fn gen_chaps_for_title_ti(ti: &TitleInfo, cr: ClipRecut, empty_title: bool) -> (String, bool) {
    let mut stra = String::new();
    stra += ";FFMETADATA1";

    let mut did_sth = false;

    for c in &ti.chapters {
        if c.start >= cr.offset
        /*(c.start >= cr.offset)
        && if let Some(e) = cr.duration {
            c.start + c.duration <= cr.offset + e
        } else {
            true
        }*/
        {
            let trans_time = c.start - cr.offset;
            let trans_end = trans_time + c.duration;

            let trans_end = trans_end.min(cr.duration.or(Some(u64::MAX)).unwrap());

            if trans_time < cr.duration.or(Some(u64::MAX)).unwrap() {
                let chap = create_chapter(
                    trans_time as u64 / 90,
                    trans_end as u64 / 90,
                    &(if empty_title {
                        format!("")
                    } else {
                        format!("# {}", c.idx)
                    }),
                );
                stra += &chap;
                did_sth = true;
            }
        }
    }
    (stra, did_sth)
}

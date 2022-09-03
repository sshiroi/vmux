use crate::{
    handling::{Bdrom, TitleId},
    *,
};

type FullTitleResult = (Vec<VSource>, Vec<Vec<ASource>>);

#[derive(Clone)]
pub struct CachedAV {
    pub video: VSource,
    pub audios: Vec<ASource>,
    pub first_pts: u64,
}
pub struct BDAVStreamCache {
    bdrom: Bdrom,
    bd_index_dir: String,
    streams: Vec<(String, CachedAV)>,

    titles: Vec<(TitleId, FullTitleResult)>,
}

pub fn avsource_from_bd_strm(bdrom: &Bdrom, strm: &str, bd_index_dir: &str) -> AVSource {
    let bdcliop = bdrom.find_stream_file(&strm);
    let bdidx = bdrom.index_for_stream(&strm, bd_index_dir);

    let bdcliop = bdcliop.to_str().unwrap();

    AVSource::from_ffms2(bdcliop, bdidx)
}
//TODO: this is a mess clean up
impl BDAVStreamCache {
    pub fn new(bdrom: &Bdrom, bd_index_dir: &str) -> BDAVStreamCache {
        BDAVStreamCache {
            bdrom: bdrom.clone(),
            bd_index_dir: bd_index_dir.to_owned(),
            streams: Vec::new(),
            titles: Vec::new(),
        }
    }

    //There is no reasoan to cache streams and titles seperatly im just lazy
    pub fn singular_stream(&mut self, cipp: &str) -> CachedAV {
        let fnd = self.streams.iter().find(|e| e.0 == cipp);
        if fnd.is_none() {
            let avsrc = avsource_from_bd_strm(&self.bdrom, cipp, &self.bd_index_dir);

            self.submit_singular_stream(cipp, avsrc);
            self.singular_stream(cipp)
        } else {
            fnd.unwrap().1.clone()
        }
    }

    pub fn index_stream_from_title(&mut self, sidx: u64, ti: &TitleInfo) -> CachedAV {
        let sddsf = ti.clips[sidx as usize].clip_id_as_str();

        let strm = self.singular_stream(&sddsf);
        //(strm.video.clone(),strm.audios.clone())
        strm.clone()
    }

    //There is no reasoan to cache streams and titles seperatly im just lazy
    pub fn submit_singular_stream(&mut self, cipp: &str, avsrc: AVSource) {
        let fnd = self.streams.iter().find(|e| e.0 == cipp);
        if fnd.is_none() {
            let first_pts = {
                let asd = avsrc.shared.lock().unwrap();
                asd.first_pts
            };
            self.streams.push((
                cipp.to_owned(),
                CachedAV {
                    video: avsrc.video(),
                    audios: avsrc.audios(),
                    first_pts,
                },
            ));
        } else {
            panic!("submit alreaady there");
        }
    }

    pub fn merged_title(
        &mut self,
        bd: &Bdrom,
        tidx: TitleId,
        ti: &TitleInfo,
        bd_index_dir: &str,
    ) -> (VSource, Vec<ASource>) {
        let _ = self.full_title_cached(tidx, bd, ti, bd_index_dir);
        let (_, (vids, auds)) = self.titles.iter().find(|e| e.0 == tidx).unwrap();

        assert_eq!(vids.len(), auds.len());

        let mut minmax_audio_cnt = auds[0].len();
        for i in 0..vids.len() {
            minmax_audio_cnt = usize::min(auds[i].len(), minmax_audio_cnt);
        }

        println!("min audio cnt: {}", minmax_audio_cnt);

        let mut auds: Vec<Vec<ASource>> = auds
            .iter()
            .map(|e| {
                let mut ss = Vec::new();
                for i in 0..minmax_audio_cnt {
                    ss.push(e[i].clone());
                }
                ss
            })
            .collect();
        auds.retain(|e| e.len() != 0);

        let combi = vids[0].clone().list(vids.clone());
        //let combi_a = auds[0].clone().list(auds);

        let mut combi_a = Vec::new();
        for a in 0..minmax_audio_cnt {
            let mut es = Vec::new();
            for aa in &auds {
                es.push(aa[a].clone())
            }
            combi_a.push(es[0].clone().list(es.clone()));
        }

        //let combi_a = auds.iter().map(|e| e[0].clone().list(e.to_vec())).collect();

        (combi, combi_a)
    }

    pub fn full_title_cached(
        &mut self,
        tidx: TitleId,
        bd: &Bdrom,
        ti: &TitleInfo,
        bd_index_dir: &str,
    ) -> FullTitleResult {
        let fnd = self.titles.iter().find(|e| e.0 == tidx);
        if let Some(e) = fnd {
            return e.1.clone();
        } else {
            let res = self.full_title(bd, ti, bd_index_dir);
            self.titles.push((tidx, res));
            self.full_title_cached(tidx, bd, ti, bd_index_dir)
        }
    }

    fn full_title(&mut self, bd: &Bdrom, ti: &TitleInfo, bd_index_dir: &str) -> FullTitleResult {
        let _bd = bd;
        let _bd_index_dir = bd_index_dir;

        let mut vids = vec![];
        let mut auds = vec![];

        for c in &ti.clips {
            let cipp = c.clip_id_as_str();
            let mut strm = self.singular_stream(&cipp);

            let clip_start = c.in_time - strm.first_pts;
            let clip_end = c.out_time - strm.first_pts;

            //println!(
            //    "start_time: {} in:{} out: {}",
            //    c.start_time, c.in_time, c.out_time
            //);

            let vid_cut_start = ((clip_start as f64 / 90_000.0) / strm.video.frame_time) as u64;
            let vid_cut_end = ((clip_end as f64 / 90_000.0) / strm.video.frame_time) as u64;

            vids.push(strm.video.cut(vid_cut_start, vid_cut_end - vid_cut_start));
            //  vids.push(strm.video);

            let mut clips_auds = Vec::new();

            for a in strm.audios {
                let aud_cut_start = ((clip_start as f64 / 90_000.0) * a.sample_rate as f64) as u64;
                let aud_cut_end = ((clip_end as f64 / 90_000.0) * a.sample_rate as f64) as u64;

                //         clips_auds.push(a);
                clips_auds.push(a.cut(aud_cut_start, aud_cut_end - aud_cut_start));
            }

            auds.push(clips_auds);
        }
        (vids, auds)
    }
}

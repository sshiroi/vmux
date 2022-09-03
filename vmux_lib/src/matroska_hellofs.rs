use crate::fs::emu_folder_builder::HelloFSFolderBuilder;
use crate::{
    bd_stream_av_cache::*, fs::ino_allocator::InoAllocator, matroska_backed::MatroskaBacked,
};

use crate::bd_cache::{BDsCache, TitleInfoProvider};
use crate::handling::*;
use crate::matroska::*;
use crate::*;
use std::sync::{Arc, Mutex};

pub fn weed_out_inexsitance(cfg: &Config, fldr: &str, e: &mut Vec<RemuxFolderEntrie>) {
    e.retain(|f| match f {
        RemuxFolderEntrie::SingularFile(e) => {
            let exsts = cfg.exists_bd(&e.src);
            if !exsts {
                println!(
                    "Removed {} from {} because bdmv {} does not exist",
                    e.name, fldr, e.src
                );
            }
            exsts
        }
        RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e) => {
            let exsts = cfg.exists_bd(&e.src);
            if !exsts {
                println!(
                    "Removed {} from {} because bdmv {} does not exist",
                    e.name, fldr, e.src
                );
            }
            exsts
        }
    });
}
pub fn flatten_remux_folder_entries(
    cfg: &Config,
    e: &[RemuxFolderEntrie],
    bdbd: &mut BDsCache,
) -> Vec<SingularRemuxMatroskaFile> {
    let mut sglr = Vec::new();

    for f in e {
        match f {
            RemuxFolderEntrie::SingularFile(e) => sglr.push(e.clone()),
            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(e) => {
                let bdrom = cfg.bluray(&e.src);
                if bdrom.is_none() {
                    continue;
                }
                let bdrom = bdrom.unwrap();

                let bd = bdbd.get_tis(bdrom);
                if bd.is_none() {
                    continue;
                }
                let bd = bd.unwrap();

                //TODO: use cached
                let ti = bd.get_titleinfo_playlist(e.playlist);
                if ti.is_none() {
                    continue;
                }
                let ti = ti.unwrap();

                for (i, _) in ti.clips.iter().enumerate() {
                    if e.max_cnt > 0 && i >= e.max_cnt as usize {
                        break;
                    }
                    let nam = e.name.to_owned();
                    let mut epnum = format!("{}", e.format_start as u64 + i as u64);
                    while epnum.len() < e.format_minwidth as usize {
                        epnum = format!("0{}", epnum);
                    }
                    let nam = nam.replace("{}", &epnum);

                    sglr.push(SingularRemuxMatroskaFile::flatten_only(
                        nam,
                        e.src.clone(),
                        BlurayExtract::PlaylistClipIndex(PlaylistClipIndex::new(
                            e.playlist,
                            i as _,
                            //e.playlist, i as _,
                            //TODO: CRIMINAM
                        )),
                    ));
                }
            }
        }
    }
    sglr
}

pub fn prepare_check_entries(
    cfg: &Config,
    fldr: &str,
    e: &Vec<RemuxFolderEntrie>,
    bdbd: &mut BDsCache,
) -> Vec<SingularRemuxMatroskaFile> {
    let mut ee = e.clone();
    weed_out_inexsitance(&cfg, fldr, &mut ee);
    let sglr = flatten_remux_folder_entries(&cfg, &ee, bdbd);
    sglr
}

pub fn mkvs_from_remux_folder(
    ino: &mut InoAllocator,
    folder: &RemuxFolder,
    cfg: &Config,
    builder: HelloFSFolderBuilder,
) -> HelloFSFolderBuilder {
    let mut builder = builder;

    let mut bdbd = BDsCache::new();
    let ee = prepare_check_entries(&cfg, &folder.name, &folder.entries, &mut bdbd);
    let mut bdbd = cache_all_streams(&cfg, bdbd, &ee);

    for e in &ee {
        builder = build_singular_remux_file(ino, builder, &folder.file_prefix, e, cfg, &mut bdbd)
    }
    builder
}

fn cache_all_streams(cfg: &Config, bdbd: BDsCache, asd: &[SingularRemuxMatroskaFile]) -> BDsCache {
    let bdbd = Arc::new(Mutex::new(bdbd));

    let mut thrd_handls = Vec::new();

    for f in asd {
        let (src, playlist_id) = {
            let e = f;

            let p = match e.extract.clone() {
                BlurayExtract::PlaylistFull(t) => t,
                BlurayExtract::PlaylistFromToChap(t, _, _) => t,
                BlurayExtract::PlaylistClipIndex(t) => t.playlist,
            };

            (e.src.to_string(), p)
        };

        //  let mut bdbd = bdbd.lock().unwrap();
        //  let bdrom = cfg.bluray(&src);
        //  let bd = bdbd.get(&bdrom, &cfg.bd_index_dir);

        let mut strms: Vec<String> = Vec::new();
        {
            let mut bdbd = bdbd.lock().unwrap();
            let bdrom = cfg.bluray(&src).unwrap();
            let bd = bdbd.get_tis(&bdrom).unwrap();

            for c in &bd.depre_pis(playlist_id).clips {
                strms.push(c.clip_id_as_str().to_string());
            }
        }
        strms.dedup();
        for s in strms {
            println!("stream: {}", s);
            //let bdbd = bdbd.lock().unwrap();
            let bdbd = bdbd.clone();

            let cipp = s;
            let bdrom = cfg.bluray(&src).unwrap();
            let bdrom = bdrom.clone();
            let bd_index_dir = cfg.bd_index_dir.clone();

            let hdndl = std::thread::spawn(move || {
                let avsrc = avsource_from_bd_strm(&bdrom, &cipp, &bd_index_dir);
                {
                    let mut bdbd = bdbd.lock().unwrap();
                    let uuu = bdbd.get_full(&bdrom, &bd_index_dir).unwrap();
                    let mut bd = uuu.lock().unwrap();
                    bd.bdav_helper.submit_singular_stream(&cipp, avsrc);
                }
            });
            thrd_handls.push(hdndl);
        }
    }
    println!("Reading start");
    for f in thrd_handls {
        f.join().unwrap();
    }
    println!("Reading end");
    let asds = Arc::try_unwrap(bdbd);
    let asds = asds.ok().expect("as");
    let asds = asds.into_inner().unwrap();

    asds
}

pub fn build_singular_remux_file(
    ino: &mut InoAllocator,
    builder: HelloFSFolderBuilder,
    prefix: &str,
    e: &SingularRemuxMatroskaFile,
    cfg: &Config,
    bdbd: &mut BDsCache,
) -> HelloFSFolderBuilder {
    let backing = build_singular_matroska_backed(e, cfg, bdbd);
    let matroksksks = build_singular_remux_file_name(prefix, e);
    println!("Building {}", matroksksks);
    let builder = builder.matroska(ino, &matroksksks, backing);
    println!("Mtrksa finbished!");
    builder
}

pub fn build_singular_remux_file_name(prefix: &str, e: &SingularRemuxMatroskaFile) -> String {
    let matroksksks = format!("{}{}.mkv", prefix, e.name);
    matroksksks
}

fn matroska_chapters_from_title(
    ti: &crate::TitleInfo,
    tcmnt: &TitleComment,
) -> Vec<MatroksChapter> {
    matroska_chapters_from_title_ex(ti, tcmnt, 0, u64::MAX)
    /*
    let mut va = vec![];
    for c in &ti.chapters {
        va.push(MatroksChapter {
            title: None,
            time_start: ((c.start as f32 / 90_000.0) * 1_000_000_000.0) as u64,
        });
    }
    va
    */
}
fn matroska_chapters_from_title_ex(
    ti: &crate::TitleInfo,
    tcmnt: &TitleComment,
    offset: u64,
    end: u64,
) -> Vec<MatroksChapter> {
    let mut va = vec![];
    for c in &ti.chapters {
        if c.start >= offset && c.start < end {
            let chptmcmt = tcmnt.get_chapter_comment(c.idx as _);
            dbg!(&chptmcmt);

            //TODO: dont hardcode bluray timescale
            va.push(MatroksChapter {
                title: chptmcmt,
                time_start: (((c.start - offset) as f32 / 90_000.0) * 1_000_000_000.0) as u64,
            });
        }
    }
    va
}

pub fn build_singular_matroska_backed(
    e: &SingularRemuxMatroskaFile,
    cfg: &Config,
    bdbd: &mut BDsCache,
) -> MatroskaBacked {
    println!("singular");
    let bdrom = cfg.bluray(&e.src).unwrap();
    let bd = bdbd.get_full(&bdrom, &cfg.bd_index_dir).unwrap();
    let mut bdd = bd.lock().unwrap();

    let chapts;

    let (mut v, a) = match e.extract.clone() {
        BlurayExtract::PlaylistFull(pi) => {
            let ti = bdd.tis_from_pis(pi).unwrap();
            let t = bdd.depre_pis(pi).clone();
            let tmnt = bdrom.getcreate_title_comment(pi);

            chapts = Some(matroska_chapters_from_title(&t, &tmnt));

            let (combi, combi_a) = bdd
                .bdav_helper
                .merged_title(&bdrom, ti, &t, &cfg.bd_index_dir);
            (combi.clone(), combi_a)
        }
        BlurayExtract::PlaylistFromToChap(pi, cha, chb) => {
            let ti = bdd.tis_from_pis(pi).unwrap();
            let t = bdd.depre_pis(pi).clone();
            let tmnt = bdrom.getcreate_title_comment(pi);

            let (mut combi, combi_a) =
                bdd.bdav_helper
                    .merged_title(&bdrom, ti, &t, &cfg.bd_index_dir);
            let (v, a, chs) = split_title_from_to_chapter(&mut combi, combi_a, &t, &tmnt, cha, chb);
            chapts = chs;

            (v, a)
        }
        BlurayExtract::PlaylistClipIndex(tit) => {
            let idx = tit.clip;
            let tmnt = bdrom.getcreate_title_comment(tit.playlist);

            println!("clp index");

            let t = bdd.depre_pis(tit.playlist).clone();

            let clpp = &t.clips[idx as usize];

            let offset = clpp.start_time;
            let end = offset + (clpp.out_time - clpp.in_time);

            chapts = Some(matroska_chapters_from_title_ex(&t, &tmnt, offset, end));

            let rr = bdd.bdav_helper.index_stream_from_title(idx, &t);
            let exprts = {
                match tit.audio_mode {
                    AudioMode::Auto => rr.audios,
                    AudioMode::Single(e) => vec![rr.audios[e as usize].clone()],
                    AudioMode::Multi(e) => {
                        let mut asd = Vec::new();
                        for i in e {
                            asd.push(rr.audios[i as usize].clone());
                        }
                        asd
                    }
                }
                //  if tit.audios == 0 {
                // vec![rr.audios[0].clone()]
                //  } else {
                //      rr.audios[0..tit.audios as usize].to_owned()
                //  }
            };
            (rr.video, exprts)
        }
    };
    if let Some(ch) = chapts {
        MatroskaBacked::new_ex(&mut v, &a, Some(&ch))
    } else {
        MatroskaBacked::new(&mut v, &a)
    }
}

fn split_title_from_to_chapter(
    combi: &mut VSource,
    combiaaa: Vec<ASource>,

    t: &TitleInfo,
    tcmnt: &TitleComment,

    cha: u64,
    chb: u64,
) -> (VSource, Vec<ASource>, Option<Vec<MatroksChapter>>) {
    let cha = &t.chapters[cha as usize];

    let starta = cha.start as f64 / 90_000.0;
    let chb = &t.chapters[chb as usize];
    let endb = (chb.start + chb.duration) as f64 / 90_000.0;

    let chapts = Some(matroska_chapters_from_title_ex(
        &t,
        tcmnt,
        cha.start,
        chb.start + chb.duration,
    ));

    let start_av = starta / combi.frame_time;
    let end_bv = endb / combi.frame_time;

    let v = combi.cut(start_av as u64, (end_bv - start_av) as u64);

    let mut asss = Vec::new();
    for combi_a in combiaaa {
        let start_aa = starta * combi_a.sample_rate as f64;
        let end_ba = endb * combi_a.sample_rate as f64;
        let a = combi_a.cut(start_aa as u64, (end_ba - start_aa) as u64);
        asss.push(a);
    }

    (v, asss, chapts)
}

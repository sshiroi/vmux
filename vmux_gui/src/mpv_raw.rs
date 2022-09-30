use std::path::Path;
use std::path::PathBuf;

use bluray_support::TitleInfo;
use tempfile::TempDir;
use vmux_lib::bd_cache::RGBDsCache;
use vmux_lib::bd_cache::TitleInfoProvider;
use vmux_lib::gen_chaps_for_title_ti;
use vmux_lib::handling::Bdrom;
use vmux_lib::handling::Config;
use vmux_lib::handling::RemuxFolder;
use vmux_lib::handling::RemuxFolderEntrie;
use vmux_lib::handling::SingularRemuxMatroskaFile;
use vmux_lib::matroska_hellofs::flatten_remux_folder_entries;
use vmux_lib::ClipRecut;

fn mpv_edl_escape(a: &str) -> String {
    format!("%{}%{}", a.len(), a)
}

fn build_edl_full<A: AsRef<Path>>(
    tmpd: A,
    tif: &TitleInfo,
    blr: &Bdrom,
    title: &str,
    bdmv_relative: bool,
    chapter_seperate: bool,
    edl_fix_offset: f64,
) -> String {
    build_edl_full_chapter_edge(
        tmpd,
        tif,
        blr,
        title,
        bdmv_relative,
        chapter_seperate,
        edl_fix_offset,
        0,
        tif.chapters.len() as u64 - 1,
    )
}

fn build_edl_full_chapter_edge<A: AsRef<Path>>(
    tmpd: A,
    tif: &TitleInfo,
    blr: &Bdrom,
    title: &str,
    bdmv_relative: bool,
    chapter_seperate: bool,
    edl_fix_offset: f64,
    a: u64,
    b: u64,
) -> String {
    let start = tif.chapters[a as usize].start;
    let end = tif.chapters[b as usize].start + tif.chapters[b as usize].duration;
    build_edl_ex(
        tmpd,
        tif,
        blr,
        title,
        bdmv_relative,
        chapter_seperate,
        edl_fix_offset,
        start,
        end,
    )
}
fn build_edl_ex<A: AsRef<Path>>(
    tmpd: A,
    tif: &TitleInfo,
    blr: &Bdrom,
    title: &str,
    bdmv_relative: bool,
    chapter_seperate: bool,
    edl_fix_offset: f64,
    a: u64,
    b: u64,
) -> String {
    let mut edl = String::new();
    edl += "# mpv EDL v0\n";
    edl += &format!("!track_meta,title={}\n", mpv_edl_escape(title));
    edl += "!no_chapters\n";

    let start = a;
    let end = b;

    let chapter_dir = tmpd.as_ref().join("chapters");
    if chapter_seperate {
        std::fs::create_dir_all(&chapter_dir).unwrap();
    }
    for c in &tif.clips {
        let duation = c.out_time - c.in_time;
        if c.start_time + duation <= start {
            continue;
        }
        if c.start_time >= end {
            continue;
        }
        let clip_id = c.clip_id_as_str();
        let clip_path_onfs = get_clip_path(&clip_id, blr);

        let stream_path = if bdmv_relative {
            let raww = clip_path_onfs
                .strip_prefix(&PathBuf::from(&blr.path))
                .unwrap();
            PathBuf::from("bdmvs")
                .join(&blr.internal_id)
                .join(raww)
                .display()
                .to_string()
        } else {
            clip_path_onfs.display().to_string()
        };

        let offset = (start as i64 - c.start_time as i64).max(0) as u64;
        let offset_to_end = (c.start_time + duation).min(end) - c.start_time;
        //For some we can use normal in_times here
        let edl_start = (offset + c.in_time) as f32 / 90_000.0;
        let edl_end = (c.in_time + offset_to_end) as f32 / 90_000.0;

        let edl_length = edl_end - edl_start;

        edl += &format!("{}", mpv_edl_escape(&stream_path));
        edl += ",";
        edl += &format!("{}", (edl_start as f64 + edl_fix_offset).max(0.0)); //no idea why this needs to be here
        edl += ",";
        edl += &format!("{}", edl_length);
        edl += &format!(",title=");
        edl += "\n";
    }
    let chpts = gen_chaps_for_title_ti(
        &tif,
        ClipRecut {
            offset: start,
            duration: Some(b - a),
        },
        true,
    );
    if chpts.1 {
        if chapter_seperate {
            let chapter_file = chapter_dir.join(format!("{}.ffmetadata", title));
            let _ = std::fs::write(&chapter_file, &chpts.0).unwrap();

            //TODO:
            //title= is so that mpv does not add filename as title
            //If we want to take titles from the gui we need to put the first one there
            edl += "!new_stream\n";
            edl += &format!(
                "{},title=\n",
                mpv_edl_escape(
                    &chapter_file
                        .strip_prefix(tmpd.as_ref())
                        .unwrap()
                        .display()
                        .to_string()
                )
            );
        } else {
            let hexify = hexify_string(&chpts.0);
            edl += "!new_stream\n";
            edl += &format!("hex://{},title=\n", hexify);
        }
    }
    return edl;
}

fn get_clip_path(clip_id: &str, blr: &Bdrom) -> PathBuf {
    let clip_palth = PathBuf::from(&blr.path);
    let clip_path_onfs = clip_palth
        .join("BDMV")
        .join("STREAM")
        .join(format!("{}.m2ts", clip_id));
    clip_path_onfs
}

pub fn handle(
    cfg: &Config,
    bdbd: &mut RGBDsCache,
    fl: &SingularRemuxMatroskaFile,
    prefix: &str,
    open_mpv: bool,
    bdmv_relative: bool,
    chapter_seperate: bool,
    edl_fix_offset: f64,
) -> Option<TempDir> {
    let fname = format!("{}{}", prefix, fl.name);
    let mut handle_chapters: Option<(TitleInfo, u64, u64, String)> = None;

    match &fl.extract {
        vmux_lib::handling::BlurayExtract::PlaylistFull(f) => {
            let blr = cfg.bluray(&fl.src).unwrap();
            let bd = bdbd.get_tis(blr).unwrap();

            let tif = bd.depre_pis(*f);

            let dir = tempfile::tempdir().unwrap();
            let _ = std::fs::create_dir_all(&dir).unwrap();
            let edl = build_edl_full(
                &dir,
                tif,
                blr,
                &fname,
                bdmv_relative,
                chapter_seperate,
                edl_fix_offset,
            );

            let edl_path = dir.path().join(format!("{}.edl", fname));

            std::fs::write(&edl_path, &edl).unwrap();
            if open_mpv {
                std::thread::spawn(move || {
                    //important code we dont want the tempdir destroyed
                    let _ = dir.path().display().to_string().len();

                    std::process::Command::new("mpv")
                        .args([format!("{}", edl_path.display().to_string())])
                        .output()
                        .expect("failed to execute process");
                });
            } else {
                return Some(dir);
            }
        }
        vmux_lib::handling::BlurayExtract::PlaylistFromToChap(ti, fr, to) => {
            let blr = cfg.bluray(&fl.src).unwrap();
            let bd = bdbd.get_tis(blr).unwrap();
            let tis = bd.depre_pis(*ti);

            handle_chapters = Some((tis.clone(), *fr, *to, fl.src.clone()));
        }
        vmux_lib::handling::BlurayExtract::PlaylistClipIndex(idx) => {
            let blr = cfg.bluray(&fl.src).unwrap();
            let bd = bdbd.get_tis(blr).unwrap();

            let tif = &bd.depre_pis(idx.playlist);

            let dir = tempfile::tempdir().unwrap();
            let _ = std::fs::create_dir_all(&dir).unwrap();

            let cipp = &tif.clips[idx.clip as usize];

            let edl = build_edl_ex(
                &dir,
                tif,
                blr,
                &fname,
                bdmv_relative,
                chapter_seperate,
                edl_fix_offset,
                cipp.start_time,
                cipp.start_time + (cipp.out_time - cipp.in_time),
            );

            let edl_path = dir.path().join(format!("{}.edl", fname));

            std::fs::write(&edl_path, &edl).unwrap();
            if open_mpv {
                std::thread::spawn(move || {
                    //important code we dont want the tempdir destroyed
                    let _ = dir.path().display().to_string().len();
                    std::process::Command::new("mpv")
                        .args([format!("{}", edl_path.display().to_string())])
                        .output()
                        .expect("failed to execute process");
                });
            } else {
                return Some(dir);
            }
        }
    };
    if let Some(e) = handle_chapters {
        let blr = cfg.bluray(&e.3).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let edl = build_edl_full_chapter_edge(
            &dir,
            &e.0,
            blr,
            &fname,
            bdmv_relative,
            chapter_seperate,
            edl_fix_offset,
            e.1,
            e.2,
        );

        std::fs::create_dir_all(&dir).unwrap();
        let edl_path = dir.path().join(format!("{}.edl", fname));

        std::fs::write(&edl_path, &edl).unwrap();

        if open_mpv {
            std::thread::spawn(move || {
                //important code we dont want the tempdir destroyed
                let _ = dir.path().display().to_string().len();

                std::process::Command::new("mpv")
                    .args([format!("{}", edl_path.display().to_string())])
                    .output()
                    .expect("failed to execute process");
            });
        } else {
            return Some(dir);
        }
    }
    return None;
}

//pub const USE_SYMLINKS: bool = false;

pub(crate) fn export_folder_as_ebl(
    cfg: &Config,
    f: &RemuxFolder,
    bdbd: &mut RGBDsCache,
    edl_fix_offset: f64,
    chapter_seperate: bool,
) {
    let pb = PathBuf::from(&cfg.mpvraw_exportlocation);

    if pb.exists() {
        let target_dir = pb.join(format!("{}", &f.name));
        if target_dir.exists() {
            for a in std::fs::read_dir(&target_dir).unwrap() {
                if let Ok(e) = a {
                    let ftype = e.file_name();
                    if ftype.to_str().unwrap().ends_with(".edl") {
                        std::fs::remove_file(e.path()).unwrap();
                    }
                }
            }
        } else {
            std::fs::create_dir_all(&target_dir).unwrap();
        }

        let target_chpt_dir = target_dir.join("chapters");
        if chapter_seperate {
            std::fs::create_dir_all(&target_chpt_dir).unwrap();
        }
        let mut srces = Vec::new();
        for f2 in &f.entries {
            match f2 {
                vmux_lib::handling::RemuxFolderEntrie::SingularFile(sglr) => {
                    let rt = handle(
                        &cfg,
                        bdbd,
                        sglr,
                        &f.file_prefix,
                        false,
                        false, // if cfg!(windows) { false } else { USE_SYMLINKS && true },
                        chapter_seperate,
                        edl_fix_offset,
                    );
                    let rt = rt.unwrap();
                    if chapter_seperate {
                        let src_chpt_dir = rt.as_ref().join("chapters");
                        //Copy chapters
                        for c in std::fs::read_dir(&src_chpt_dir).unwrap() {
                            let c = c.unwrap();
                            std::fs::copy(&c.path(), target_chpt_dir.join(c.file_name())).unwrap();
                        }
                    }

                    for c in std::fs::read_dir(&rt).unwrap() {
                        let c = c.unwrap();
                        if c.metadata().unwrap().is_file() {
                            std::fs::copy(&c.path(), target_dir.join(c.file_name())).unwrap();
                        }
                    }

                    if !srces.contains(&sglr.src) {
                        srces.push(sglr.src.to_string());
                    }
                }
                vmux_lib::handling::RemuxFolderEntrie::MultipleFilePlaylistClipSplit(_) => {
                    unreachable!()
                }
            }
        }

        //if (!cfg!(windows)) && USE_SYMLINKS {
        //    let target_bdmvs_dir = target_dir.join("bdmvs");
        //    std::fs::create_dir_all(&target_bdmvs_dir).unwrap();
        //    let pp = target_bdmvs_dir.as_path();
        //    for s in &srces {
        //        let blr = cfg.bluray(s).unwrap();
        //        let a = &blr.path;
        //        let b = pp.join(s);
        //        std::os::unix::fs::symlink(a, b).unwrap();
        //    }
        //}
    } else {
        println!("FOlder does not exist");
    }
}

pub(crate) fn sort_and_flatten(cfg: &Config, f: &mut RemuxFolder, bdbd: &mut RGBDsCache) {
    let fltn = flatten_remux_folder_entries(cfg, &f.entries, bdbd);
    f.entries = fltn
        .into_iter()
        .map(|e| RemuxFolderEntrie::SingularFile(e))
        .collect();
    f.sort_entries_name();
}

fn hexify_string(a: &str) -> String {
    let mut stra = String::new();
    //stra.reserve(a.len() * 2);
    for a in a.bytes() {
        stra += &format!("{:02x}", a);
    }
    stra
}

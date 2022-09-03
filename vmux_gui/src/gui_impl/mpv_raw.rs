use std::path::Path;
use std::path::PathBuf;

use bluray_support::TitleInfo;
use tempfile::TempDir;
use vmux_lib::bd_cache::BDsCache;
use vmux_lib::bd_cache::TitleInfoProvider;
use vmux_lib::gen_chaps_for_title_ti;
use vmux_lib::handling::Bdrom;
use vmux_lib::handling::Config;
use vmux_lib::handling::RemuxFolder;
use vmux_lib::handling::RemuxFolderEntrie;
use vmux_lib::handling::SingularRemuxMatroskaFile;
use vmux_lib::matroska_hellofs::flatten_remux_folder_entries;
use vmux_lib::ClipRecut;

use crate::egui;
use crate::GuiGlobalState;

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
    edl += &format!("!track_meta,title={}\n", title);
    // edl += "!no_chapters\n";

    let start = a;
    let end = b;

    let chapter_dir = tmpd.as_ref().join("chapters");
    std::fs::create_dir_all(&chapter_dir).unwrap();

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

        let iuiui = if bdmv_relative {
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

        edl += &format!("{}", &iuiui);
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
                chapter_file
                    .strip_prefix(tmpd.as_ref())
                    .unwrap()
                    .display()
                    .to_string()
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
    bdbd: &mut BDsCache,
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

pub fn gui_mpv_raw(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    let _ = ui;
    let _ = asd;
    /*
    egui::ScrollArea::vertical()
        .id_source("mpv_unindexed")
        .show(ui, |ui| {
            let smen = asd.mpv_raw.is_some();
            ui.label(format!("status: {}", smen));

            ui.label("Save location");
            ui.text_edit_singleline(&mut asd.vmux_config.mpvraw_exportlocation);
            ui.separator();
            if smen {
                if ui.button("Close").clicked() {
                    asd.mpv_raw = None;
                }
            }
            if let Some(folders) = &asd.mpv_raw {
                if ui.text_edit_singleline(&mut asd.mpv_raw_search).clicked() {}

                for f in &folders.clone() {
                    if !f.show{
                        continue;
                    }
                    if asd.mpv_raw_search != "" {
                        if !f.name
                            .to_lowercase()
                            .contains(&(asd.mpv_raw_search.to_owned().to_lowercase()))
                        {
                            continue;
                        }
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Export").clicked() {
                            export_folder_as_ebl(&asd.vmux_config,f,&mut asd.bdsc,asd.edl_fix_offset);
                        }
                        ui.collapsing(&f.name, |ui| {
                            for f2 in &f.entries {
                                match f2 {
                                    vmux_lib::handling::RemuxFolderEntrie::SingularFile(sglr) => {
                                        if ui.button(format!("{}{}", &f.file_prefix, sglr.name)).clicked() {
                                            handle( &asd.vmux_config, &mut asd.bdsc, sglr,&f.file_prefix,true,false,asd.edl_fix_offset);
                                        }
                                    },
                                    vmux_lib::handling::RemuxFolderEntrie::MultipleFileTitleClipSplit(_) => unreachable!(),
                                }
                            }
                        });
                    });
                }
            }else {
                if ui.button("Setup").clicked() {
                    let mut vc = asd.vmux_config.folders.clone();
                    for f in &mut vc {
                        if !f.show { continue;}
                        sort_and_flatten(&asd.vmux_config,f,&mut asd.bdsc);
                    }
                    vc.sort_by(|a,b| a.file_prefix.partial_cmp(&b.file_prefix).unwrap());


                    asd.mpv_raw = Some(vc);
                }
            }


        });
        */
}

//pub const USE_SYMLINKS: bool = false;

pub(crate) fn export_folder_as_ebl(
    cfg: &Config,
    f: &RemuxFolder,
    bdbd: &mut BDsCache,
    edl_fix_offset: f64,
    chapter_seperate: bool,
) {
    let pb = PathBuf::from(&cfg.mpvraw_exportlocation);

    if pb.exists() {
        let target_dir = pb.join(format!("{}", &f.name));
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir).unwrap();
            std::fs::create_dir_all(&target_dir).unwrap();
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

pub(crate) fn sort_and_flatten(cfg: &Config, f: &mut RemuxFolder, bdbd: &mut BDsCache) {
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

/*
old code
    let mut nfo = Vec::new();
    for c in &tif.clips {
        let asdd = process_clipp(&bd.bd,*f,c,blr);
        nfo.push(asdd);
    }

    let mut stras = Vec::new();

    stras.push(format!("mpv"));
    for f in &nfo {
        stras.push(format!("--{{"));
        stras.push(format!("{}",f.0.display().to_string()));
        stras.push(format!("--chapters-file={}",f.1.display().to_string()));
        stras.push(format!("--}}"));
    }
    let nfos: Vec<TempDir> = nfo.into_iter().map(|e| e.2).collect();

    std::thread::spawn(move || {
        //important code we dont want the tempdir destroyed
        let dd = nfos;
        let _ = dd.len();

        std::process::Command::new("mpv")
            .args(
                stras
            )
            .output()
            .expect("failed to execute process");
    });
}





fn process_clipp(bd: &BD, title_idx: u64, cipp: &Clip, blr: &Bdrom) -> (PathBuf, PathBuf, TempDir) {
    let clip_id = cipp.clip_id_as_str();
    let chapters = gen_chaps_for_title(
        &bd,
        title_idx as u32,
        vmux_lib::ClipRecut {
            offset: cipp.start_time,
            duration: Some(cipp.out_time - cipp.in_time),
        },
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(&dir).unwrap();

    let file_path = dir.path().join(format!("asd{}.ffmetadata", clip_id));
    dbg!(&file_path);
    let _ = std::fs::write(&file_path, chapters).unwrap();

    let clip_path_onfs = get_clip_path(&clip_id, blr);

    (clip_path_onfs, file_path, dir)
}


fn build_edl_full<A: AsRef<Path>>(tmpd: A,tif: &TitleInfo,blr: &Bdrom,title: &str) -> String{
    let mut edl = String::new();
    edl += "# mpv EDL v0\n";
    edl += format!("!track_meta,title={}\n",title);
    edl += "!no_chapters\n";


    let chapter_dir = tmpd.as_ref().join("chapters");
    std::fs::create_dir_all(&chapter_dir).unwrap();

    for c in &tif.clips {
        let clip_id = c.clip_id_as_str();
        let clip_path_onfs = get_clip_path(&clip_id,blr);
        let iuiui = clip_path_onfs.display().to_string();
        //let aa = find_infooo(iuiui);
        //For some we can use normal in_times here
        let edl_start = c.in_time as f32 / 90_000.0;
        let edl_end   = c.out_time as f32 / 90_000.0;

        let edl_length = edl_end - edl_start;

        let chpts = gen_chaps_for_title_ti(&tif, ClipRecut {
            offset: c.start_time,
            duration: Some(c.out_time-c.in_time),
        });
        let chapter_file = chapter_dir.join(format!("{}_t{}.ffmetadata",blr.internal_id,tif.idx));
        let _ = std::fs::write(&chapter_file,&chpts).unwrap();


        edl += &format!("{}",&iuiui);
        edl += ",";
        edl += &format!("{}",edl_start);
        edl += ",";
        edl += &format!("{}",edl_length);
        edl += "\n";
        edl += "!new_stream\n";
        edl += &format!("{}\n",chapter_file.display().to_string());
    }
    return edl;
}



fn find_infooo(media_file: &str) -> (u64, f64) {
    let outpt = std::process::Command::new("ffprobe")
        .args([
            format!("{}", media_file),
            format!("-v"),
            format!("error"),
            format!("-show_format"),
            format!("-show_streams"),
        ])
        .output()
        .expect("failed to execute process");
    let asd = String::from_utf8(outpt.stdout).unwrap();

    let bf = BufReader::new(Cursor::new(asd));
    let mut pts = 0u64;
    let mut start = 0.0;
    for l in bf.lines() {
        let l = l.unwrap();
        if l.contains("start_pts=") {
            let aa: Vec<&str> = l.split("=").collect();
            pts = aa[1].parse().unwrap();
        }
        if l.contains("start_time=") {
            let aa: Vec<&str> = l.split("=").collect();
            start = aa[1].parse().unwrap();
        }
    }

    (pts, start)
}
*/

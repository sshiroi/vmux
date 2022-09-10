use crate::egui;
use std::{io::BufRead, process::Stdio};

use crate::gui_common::*;

use bluray_support::TitleInfo;
use vmux_lib::{format_duration, handling::*};

use super::VlcOutput;

pub fn gui_bd_inspection(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    let mut do_close = false;
    egui::ScrollArea::vertical()
        .id_source("inspec")
        .show(ui, |ui| {
            if asd.inspect_bd.is_none() {
                ui.label("No bdmv selected for inspection");
                return;
            }
            let (id, info) = asd.inspect_bd.as_ref().unwrap();

            ui.horizontal(|ui| {
                if ui.button("Open in VLC").clicked() {
                    let pp = info.path.clone();
                    let aaa = asd.vlc_output.clone();

                    {
                        let mut lck = aaa.lock().unwrap();
                        *lck = Some(VlcOutput {
                            inspect_bdromid: id.to_owned(),
                            ..Default::default()
                        })
                    }
                    std::thread::spawn(move || {
                        let mut child = std::process::Command::new("vlc")
                            .args([format!("bluray:///{}", pp)])
                            .env("BD_DEBUG_MASK", "64")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("failed to execute process");
                        println!("VLC EXIT");

                        // let stdout = child.stdout.take().expect("failed to get stdin");
                        //shits on stderr for some reaason
                        let stderr = child.stderr.take().expect("failed to get stdin");
                        std::thread::spawn(move || {
                            let bfr = std::io::BufReader::new(stderr);

                            for l in bfr.lines() {
                                let l = l.unwrap();
                                if !l.contains("HDMV") {
                                    continue;
                                }
                                let mut lck = aaa.lock().unwrap();
                                let lck = lck.as_mut().unwrap();
                                super::vlc::vlc_process_line(lck, &l);
                                lck.raw_lines.push(l);
                            }
                        });
                    });
                }
                if ui.button("Close Inspect").clicked() {
                    do_close = true;
                }
                ui.checkbox(&mut asd.inspect_longpath, "Longpath");
            });

            let amo = &mut asd.bd_inspect_sortmode;
            egui::ComboBox::from_label("SortMode")
                .selected_text(format!("{:?}", amo))
                .show_ui(ui, |ui| {
                    ui.selectable_value(amo, BdInspectSortMode::Title, "Title");
                    ui.selectable_value(amo, BdInspectSortMode::Playlist, "Playlist");
                });

            ui.heading("Titles");
            let mut cpy: Vec<(TitleId, TitleInfo)> = info
                .legacy_title_list
                .clone()
                .iter()
                .enumerate()
                .map(|e| (TitleId::from_title_id(e.0 as u64), e.1.clone()))
                .collect();

            match asd.bd_inspect_sortmode {
                BdInspectSortMode::Title => {}
                BdInspectSortMode::Playlist => {
                    cpy.sort_by(|a, b| (&a.1.playlist).partial_cmp(&b.1.playlist).unwrap())
                }
            };

            for (title_idx, t) in cpy {
                let playlist_i = PlaylistId::from_pis(t.playlist as u64);

                let mut title_comment = asd
                    .vmux_config
                    .bluray(id)
                    .unwrap()
                    .getcreate_title_comment(playlist_i);
                let duration = format_duration(t.duration);
                let clp_cnt = t.clip_count;
                let chpt_cnt = t.chapter_count;

                let extra_comment = format!("({})", title_comment.name);

                let strt = match asd.bd_inspect_sortmode {
                    BdInspectSortMode::Title => format!("{} Title", title_idx.acual_title_id()),
                    BdInspectSortMode::Playlist => format!("{} Mpls", t.playlist),
                };
                ui.horizontal(|ui| {
                    if asd.inspect_longpath {
                        let te = egui::TextEdit::singleline(&mut title_comment.name)
                            .desired_width(190.0)
                            .show(ui);
                        if te.response.changed() {
                            asd.vmux_config.bluray_mut(id, |bd| {
                                bd.set_title_comment(playlist_i, title_comment.clone())
                            });
                        }
                    }
                    let collapsing_header = egui::CollapsingHeader::new(format!(
                        "{} {} chp: {} clp:{} {} ",
                        strt, duration, chpt_cnt, clp_cnt, extra_comment
                    ))
                    .id_source(format!(
                        "bdmv_{}_inspect_title_{}",
                        id,
                        title_idx.acual_title_id()
                    ))
                    .show(ui, |ui| {
                        //Inside collapsing header

                        //Title comment editing
                        if ui.text_edit_singleline(&mut title_comment.name).changed() {
                            asd.vmux_config.bluray_mut(id, |bd| {
                                bd.set_title_comment(playlist_i, title_comment.clone())
                            });
                        }

                        //Playlist number + list of clips
                        ui.label(format!("PLS: {}", t.playlist));
                        ui.label(format!("TTL: {}", title_idx.acual_title_id()));
                        ui.label(format!("clips: "));

                        egui::CollapsingHeader::new(format!("Clips ({})", t.clips.len())).show(
                            ui,
                            |ui| {
                                for (clip_i, clp) in t.clips.iter().enumerate() {
                                    //let title_i = title_idx;

                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{}.m2ts: {}",
                                            clp.clip_id_as_str(),
                                            format_duration(clp.out_time - clp.in_time)
                                        ));

                                        if ui.button("Play").clicked() {
                                            let infoss: Option<_> = info
                                                .strms
                                                .iter()
                                                .find(|e| &e.0 == &clp.clip_id_as_str());

                                            let ss = infoss.unwrap().1.to_owned();
                                            std::thread::spawn(move || {
                                                std::process::Command::new("mpv")
                                                    .args([format!("{}", ss.display())])
                                                    .output()
                                                    .expect("failed to execute process");
                                            });
                                        }

                                        if let Some(ii) = &asd.selected_folder {
                                            let fld = &mut asd.vmux_config.folders[*ii];
                                            if ui.button("Add ClipIndex").clicked() {
                                                fld.entries.push(RemuxFolderEntrie::SingularFile(
                                                    SingularRemuxMatroskaFile::new(
                                                        title_comment.name.clone(),
                                                        id.to_string(),
                                                        BlurayExtract::PlaylistClipIndex(
                                                            PlaylistClipIndex::new(
                                                                playlist_i,
                                                                clip_i as u64,
                                                            ),
                                                        ),
                                                    ),
                                                ));
                                                fld.sort_entries();
                                            }
                                        }
                                    });
                                }
                            },
                        );

                        egui::CollapsingHeader::new(format!("Marks ({})", t.marks.len())).show(
                            ui,
                            |ui| {
                                ui.label(format!("idx clip start duration"));
                                for m in &t.marks {
                                    ui.label(format!(
                                        "{:02} {:05} {} {}",
                                        m.idx,
                                        m.clip_ref,
                                        format_duration(m.start),
                                        format_duration(m.duration)
                                    ));
                                }
                            },
                        );

                        egui::CollapsingHeader::new(format!("Chapters ({})", t.chapters.len()))
                            .show(ui, |ui| {
                                ui.label(format!("idx clip start duration"));
                                for (i, m) in t.chapters.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{:02} {:05} {} {}",
                                            m.idx,
                                            m.clip_ref,
                                            format_duration(m.start),
                                            format_duration(m.duration)
                                        ));

                                        let mut asd2 = if let Some(e) =
                                            title_comment.get_chapter_comment(m.idx as _)
                                        {
                                            e
                                        } else {
                                            String::new()
                                        };
                                        if ui.text_edit_singleline(&mut asd2).changed() {
                                            title_comment.set_chapter_comment(m.idx as _, &asd2);
                                            asd.vmux_config.bluray_mut(id, |bd| {
                                                bd.set_title_comment(
                                                    playlist_i,
                                                    title_comment.clone(),
                                                )
                                            });
                                        }
                                        if let Some(ii) = &asd.selected_folder {
                                            if ui.button("Add").clicked() {
                                                let fld = &mut asd.vmux_config.folders[*ii];
                                                //TODO: add singular chapter
                                                fld.entries.push(RemuxFolderEntrie::SingularFile(
                                                    SingularRemuxMatroskaFile::new(
                                                        asd2.clone(),
                                                        id.to_string(),
                                                        BlurayExtract::PlaylistFromToChap(
                                                            playlist_i, i as u64, i as u64,
                                                        ),
                                                    ),
                                                ));
                                            }
                                        }
                                    });
                                }
                            });

                        //Play title in mpv
                        if ui.button("Play").clicked() {
                            let bd = asd.vmux_config.bluray(id).unwrap();

                            let (a, b) = (t.playlist, bd.path.clone());
                            std::thread::spawn(move || {
                                std::process::Command::new("mpv")
                                    .args([
                                        format!("bluray://mpls/{}", a),
                                        format!("-bluray-device={}", b),
                                    ])
                                    .output()
                                    .expect("failed to execute process");
                            });
                        }

                        if let Some(ii) = &asd.selected_folder {
                            let fld = &mut asd.vmux_config.folders[*ii];
                            if ui.button("Add FullTitle").clicked() {
                                fld.entries.push(RemuxFolderEntrie::SingularFile(
                                    SingularRemuxMatroskaFile::new(
                                        title_comment.name.clone(),
                                        id.to_string(),
                                        BlurayExtract::PlaylistFull(playlist_i),
                                    ),
                                ));
                                fld.sort_entries();
                            }
                            if ui.button("Add ClipSplit").clicked() {
                                fld.entries
                                    .push(RemuxFolderEntrie::MultipleFilePlaylistClipSplit(
                                        ClipSplit {
                                            name: title_comment.name.clone(),
                                            src: id.to_string(),
                                            playlist: playlist_i,
                                            format_start: 1,
                                            format_minwidth: 2,
                                            max_cnt: 0,
                                        },
                                    ));
                            }
                        }
                    });

                    if let Some(ii) = &asd.selected_folder {
                        let fld = &mut asd.vmux_config.folders[*ii];

                        if collapsing_header.header_response.secondary_clicked() {
                            fld.entries.push(RemuxFolderEntrie::SingularFile(
                                SingularRemuxMatroskaFile::new(
                                    title_comment.name.clone(),
                                    id.to_string(),
                                    BlurayExtract::PlaylistFull(playlist_i),
                                ),
                            ));
                            fld.sort_entries();
                        }
                    }
                });
            }

            asd.vmux_config.bluray_mut(id, |mtt| {
                ui.text_edit_multiline(&mut mtt.general_comment);
            });
        });
    if do_close {
        asd.inspect_bd = None;
    }
}

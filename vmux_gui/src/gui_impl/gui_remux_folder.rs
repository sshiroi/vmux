use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::egui;
use vmux_lib::bd_cache::{BDsCache, TitleInfoProvider};
use vmux_lib::handling::*;

fn add_button(ui: &mut egui::Ui, folder: &mut RemuxFolder, zz: &mut RemuxFolderEntrie) {
    egui::ComboBox::from_label("")
        .selected_text(format!(
            "{}",
            match zz {
                RemuxFolderEntrie::SingularFile(_) => "SingularFile",
                RemuxFolderEntrie::MultipleFilePlaylistClipSplit(_) => "ClipSplit",
            }
        ))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                zz,
                RemuxFolderEntrie::SingularFile(SingularRemuxMatroskaFile::default()),
                "SingularFile",
            );
            ui.selectable_value(
                zz,
                RemuxFolderEntrie::MultipleFilePlaylistClipSplit(ClipSplit::default()),
                "ClipSplit",
            );
        });
    if ui.button("Add").clicked() {
        folder.entries.push(zz.clone());
    }
}

fn find_errors(cfg: &Config, bdbd: &mut BDsCache, iiiii: usize) -> Vec<(usize, String)> {
    let mut errors = Vec::new();
    for f in cfg.folders[iiiii].entries.iter().enumerate() {
        let bdrom = cfg.bluray(f.1.src());
        if bdrom.is_none() {
            errors.push((f.0, "Src does not exists".to_owned()));
            continue;
        }
        let src = bdbd.get_full(bdrom.unwrap(), &cfg.bd_index_dir);

        if src.is_none() {
            errors.push((
                f.0,
                format!("BDMV path does not exist {}", bdrom.unwrap().path),
            ));
            continue;
        }
        let src = src.unwrap();
        let cbd = src.lock().unwrap();
        let pipp = match f.1 {
            RemuxFolderEntrie::SingularFile(f) => match &f.extract {
                BlurayExtract::PlaylistFull(t) => *t,
                BlurayExtract::PlaylistFromToChap(t, _, _) => *t,
                BlurayExtract::PlaylistClipIndex(t) => t.playlist,
            },
            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(csplt) => csplt.playlist,
        };
        if cbd.get_titleinfo_playlist(pipp).is_none() {
            errors.push((
                f.0,
                format!("Paylist does not exist {}", pipp.acual_title_pis()),
            ));
            continue;
        }
        //Check chapters existance
        let chpts = match f.1 {
            RemuxFolderEntrie::SingularFile(f) => match &f.extract {
                BlurayExtract::PlaylistFull(_) => vec![],
                BlurayExtract::PlaylistClipIndex(_) => vec![],
                BlurayExtract::PlaylistFromToChap(_, a, b) => vec![*a, *b],
            },
            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(_) => {
                vec![]
            }
        };
        let ti = cbd.get_titleinfo_playlist(pipp).unwrap();
        for c in chpts {
            if c >= ti.chapters.len() as u64 {
                errors.push((f.0, format!("Out of bounds chapter {}", c)));
                continue;
            }
        }

        //Check misc logic
        match f.1 {
            RemuxFolderEntrie::SingularFile(ff) => match &ff.extract {
                BlurayExtract::PlaylistFull(_) => {}
                BlurayExtract::PlaylistClipIndex(tco) => {
                    if tco.clip >= ti.clips.len() as u64 {
                        errors.push((f.0, format!("Out of bounds clip index {}", tco.clip)));
                        continue;
                    }
                }

                BlurayExtract::PlaylistFromToChap(_, a, b) => {
                    if *b < *a {
                        errors.push((f.0, format!("order {} < {}", *b, *a)));
                        continue;
                    }
                }
            },
            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(splt) => {
                if splt.max_cnt != 0 {
                    if splt.max_cnt > ti.clips.len() as u64 {
                        errors.push((f.0, format!("Out of bounds max_cnt {}", splt.max_cnt)));
                        continue;
                    }
                }
            }
        };
    }
    errors
}

pub fn gui_remux_folder(ui: &mut egui::Ui, asd: &mut crate::GuiGlobalState) {
    egui::ScrollArea::vertical()
        .id_source("remuxxx")
        .show(ui, |ui| {
            let mut hasher = DefaultHasher::new();

            let mut vmux_config_change = false;
            if let Some(iiiii) = asd.selected_folder {
                asd.vmux_config.hash(&mut hasher);
                let hahhy = hasher.finish();

                if hahhy != asd.remux_folder_last_hash_all {
                    asd.remux_folder_errors = find_errors(&asd.vmux_config, &mut asd.bdsc, iiiii);
                    vmux_config_change = true;
                }
                asd.remux_folder_last_hash_all = hahhy;
            }

            let errors = &asd.remux_folder_errors;

            let grid_cnt = if asd.inspect_longpath { 3 } else { 2 };

            egui::Grid::new("my_grid")
                .num_columns(grid_cnt)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    if let Some(iiiii) = asd.selected_folder {
                        let folder = &mut asd.vmux_config.folders[iiiii];

                        ui.horizontal(|ui| {
                            add_button(ui, folder, &mut asd.tmp_folder_entrie);
                            //ui.end_row();
                            //ui.separator();
                            if ui.button("Sort").clicked() {
                                //                            let feee = &mut folder.entries;
                                //                            let mut fe2: Vec<&mut RemuxFolderEntrie> = feee.iter_mut().map(|e| e).collect();
                                folder.sort_entries();
                            }
                            ui.label(format!("errors: {}", errors.len()));
                        });
                        ui.end_row();
                        let mut asd_id = 0;
                        let mut to_delete = None;
                        let mut to_up = None;
                        let mut to_down = None;

                        let mut last_src = String::new();

                        for ee in &mut folder.entries {
                            if last_src != ee.src() {
                                if asd.inspect_longpath {
                                    ui.separator();
                                }
                                ui.separator();
                                ui.label(format!("bdmv:{}", ee.src()));
                                ui.end_row();
                            }
                            last_src = ee.src().to_string();
                            //    ui.horizontal(|ui|{
                            if asd.inspect_longpath {
                                let mut n = ee.name().to_string();
                                if egui::TextEdit::singleline(&mut n)
                                    .desired_width(190.0)
                                    .hint_text("Name")
                                    .show(ui)
                                    .response
                                    .changed()
                                {
                                    ee.set_name(n);
                                }
                            }

                            ee_edit(ui, ee, asd_id);
                            //     });

                            ui.horizontal(|ui| {
                                if ui.button("Delete").clicked() {
                                    to_delete = Some(asd_id);
                                }
                                if ui.button("U").clicked() {
                                    to_up = Some(asd_id);
                                }
                                if ui.button("D").clicked() {
                                    to_down = Some(asd_id);
                                }

                                for e in errors {
                                    if e.0 == asd_id {
                                        ui.label(
                                            egui::RichText::new(&e.1).color(egui::Color32::RED),
                                        );
                                    }
                                }
                            });

                            ui.end_row();
                            asd_id += 1;
                        }
                        if let Some(e) = to_delete {
                            folder.entries.remove(e);
                        }
                        if let Some(e) = to_up {
                            if e != 0 {
                                let a = folder.entries[e - 1].clone();
                                let b = folder.entries[e].clone();

                                folder.entries[e - 1] = b;
                                folder.entries[e] = a;
                            }
                            folder.sort_entries();
                        }
                        if let Some(e) = to_down {
                            if e != folder.entries.len() - 1 {
                                let a = folder.entries[e + 1].clone();
                                let b = folder.entries[e].clone();

                                folder.entries[e + 1] = b;
                                folder.entries[e] = a;
                            }
                            folder.sort_entries();
                        }

                        let mut hasher = DefaultHasher::new();
                        folder.hash(&mut hasher);
                        let hash_after = hasher.finish();

                        let mut folder_cpy = folder.clone();

                        drop(folder);

                        let new_errors = if hash_after != asd.remux_folder_lasthash {
                            find_errors(&asd.vmux_config, &mut asd.bdsc, iiiii)
                        } else {
                            errors.clone()
                        };

                        if new_errors.len() == 0
                            && (asd.remux_folder_flattend.is_none()
                                || (hash_after != asd.remux_folder_lasthash)
                                || vmux_config_change)
                        {
                            // drop(folder);
                            super::mpv_raw::sort_and_flatten(
                                &asd.vmux_config,
                                &mut folder_cpy,
                                &mut asd.bdsc,
                            );
                            asd.remux_folder_flattend = Some(folder_cpy);
                        }
                        asd.remux_folder_lasthash = hash_after;
                    } else {
                        ui.label("Currently no folder selected!");
                        ui.end_row();
                    }
                });

            let error_free = if let Some(iiiii) = asd.selected_folder {
                if errors.len() == 0 {
                    find_errors(&asd.vmux_config, &mut asd.bdsc, iiiii).len() == 0
                } else {
                    true
                }
            } else {
                false
            };
            ui.separator();
            if let Some(e) = &asd.remux_folder_flattend {
                egui::Grid::new("my_top_remux_grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.heading("Resulting files");
                        if error_free {
                            if ui.button("Export EDL").clicked() {
                                super::mpv_raw::export_folder_as_ebl(
                                    &asd.vmux_config,
                                    e,
                                    &mut asd.bdsc,
                                    asd.edl_fix_offset,
                                    false,
                                );
                            }
                        }
                        number_widgef64(ui, &mut asd.edl_fix_offset, "");

                        ui.end_row();

                        for ent in &e.entries {
                            match ent {
                                RemuxFolderEntrie::SingularFile(sf) => {
                                    ui.label(format!("{}{}", e.file_prefix, sf.name));
                                    if error_free {
                                        if ui.button("Watch").clicked() {
                                            super::mpv_raw::handle(
                                                &asd.vmux_config,
                                                &mut asd.bdsc,
                                                sf,
                                                &e.file_prefix,
                                                true,
                                                false,
                                                false,
                                                asd.edl_fix_offset,
                                            );
                                        }
                                    }
                                    ui.end_row();
                                }
                                RemuxFolderEntrie::MultipleFilePlaylistClipSplit(_) => {
                                    unreachable!()
                                }
                            }
                        }
                    });
            }
        });
}

fn ee_edit(ui: &mut egui::Ui, ee: &mut RemuxFolderEntrie, asd_id: usize) {
    match ee {
        RemuxFolderEntrie::SingularFile(f) => {
            let fscr = &mut f.src;
            let exrtrt = &mut f.extract;
            egui::CollapsingHeader::new(&f.name)
                .id_source(format!("{} fff", asd_id))
                .show(ui, |ui| {
                    ui.add(egui::TextEdit::singleline(&mut f.name).hint_text("Name"));
                    ui.add(
                        egui::TextEdit::singleline(fscr)
                            .id_source(format!("src_{}", asd_id))
                            .hint_text("src"),
                    );
                    egui::ComboBox::from_label("BlurayExtract")
                        .selected_text(format!("{}", exrtrt.brief_name()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                exrtrt,
                                BlurayExtract::PlaylistFull(PlaylistId::default()),
                                "PlaylistFull",
                            );
                            ui.selectable_value(
                                exrtrt,
                                BlurayExtract::PlaylistClipIndex(PlaylistClipIndex::new(
                                    PlaylistId::default(),
                                    0,
                                )),
                                "PlaylistClipIndex",
                            );

                            ui.selectable_value(
                                exrtrt,
                                BlurayExtract::PlaylistFromToChap(PlaylistId::default(), 0, 0),
                                "ChapFrTo",
                            );
                        });
                    match exrtrt {
                        BlurayExtract::PlaylistFull(t) => {
                            let mut tto = t.acual_title_pis();
                            number_widge(ui, &mut tto, "playlist:");
                            t.set_acual_title_pis(tto);
                        }
                        BlurayExtract::PlaylistClipIndex(tci) => {
                            let mut tto = tci.playlist.acual_title_pis();
                            number_widge(ui, &mut tto, "playlist:");
                            tci.playlist.set_acual_title_pis(tto);
                            number_widge(ui, &mut tci.clip, "index:");

                            //     ui.checkbox(&mut tci.as_full_stream, "As FullStream");

                            let amo = &mut tci.audio_mode;
                            egui::ComboBox::from_label("AudioMode")
                                .selected_text(format!("{}", amo.brief_name()))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(amo, AudioMode::Auto, "Auto");
                                    ui.selectable_value(amo, AudioMode::Single(0), "Single");

                                    ui.selectable_value(amo, AudioMode::Multi(vec![0]), "Multi");
                                });
                            match amo {
                                AudioMode::Auto => {}
                                AudioMode::Single(d) => {
                                    number_widge(ui, d, "audio:");
                                }
                                AudioMode::Multi(ds) => {
                                    let mut to_del = None;
                                    for (i, f) in ds.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}:", i));

                                            number_widge(ui, f, "");

                                            if ui.button("Del").clicked() {
                                                to_del = Some(i);
                                            }
                                        });
                                    }
                                    if ui.button("Add").clicked() {
                                        let mut a = 0;
                                        for f in ds.iter() {
                                            if *f + 1 > a {
                                                a = *f + 1;
                                            }
                                        }
                                        ds.push(a);
                                    }
                                    if let Some(d) = to_del {
                                        ds.remove(d);
                                    }
                                }
                            }
                        }
                        BlurayExtract::PlaylistFromToChap(t, fr, to) => {
                            //ui.horizontal(|ui| {
                            let mut rt = t.acual_title_pis();
                            number_widge(ui, &mut rt, "playlist:");
                            t.set_acual_title_pis(rt);
                            number_widge(ui, fr, "from:");
                            number_widge(ui, to, "to:");
                            //    });
                        }
                    }
                });
        }
        RemuxFolderEntrie::MultipleFilePlaylistClipSplit(clpslplt) => {
            egui::CollapsingHeader::new(&clpslplt.name)
                .id_source(format!("{} ss", asd_id))
                .show(ui, |ui| {
                    ui.add(egui::TextEdit::singleline(&mut clpslplt.name).hint_text("name"));

                    ui.add(egui::TextEdit::singleline(&mut clpslplt.src).hint_text("src"));
                    let mut t = clpslplt.playlist.acual_title_pis();
                    number_widge(ui, &mut t, "playlist:");
                    clpslplt.playlist.set_acual_title_pis(t);
                    number_widge(ui, &mut clpslplt.format_start, "formatstart:");
                    number_widge(ui, &mut clpslplt.max_cnt, "max_cnt:");
                    number_widge(ui, &mut clpslplt.format_minwidth, "formatminwidth:");
                });
        }
    }
}

/*
fn number_widge<'a,T: egui::emath::Numeric>(trgt: &'a mut u64, prfx: &str) -> egui::DragValue<'a> {
    let scroll_speed = 0.007;

    egui::DragValue::new(trgt).prefix(prfx).speed(scroll_speed)
}
*/

fn number_widge<'a>(ui: &mut egui::Ui, trgt: &'a mut u64, prfx: &str) {
    //    let scroll_speed = 0.007;
    //    ui.add(egui::DragValue::new(trgt).prefix(prfx).speed(scroll_speed));
    ui.horizontal(|ui| {
        ui.label(prfx);
        let mut txt = format!("{}", trgt);
        ui.text_edit_singleline(&mut txt);
        *trgt = txt.parse::<u64>().ok().or(Some(*trgt)).unwrap();
    });
}

fn number_widgef64<'a>(ui: &mut egui::Ui, trgt: &'a mut f64, prfx: &str) {
    let scroll_speed = 0.00007;
    ui.add(egui::DragValue::new(trgt).prefix(prfx).speed(scroll_speed));
    //ui.horizontal(|ui| {
    //    ui.label(prfx);
    //    let mut txt = format!("{}", trgt);
    //    ui.text_edit_singleline(&mut txt);
    //    *trgt = txt.parse::<f64>().ok().or(Some(*trgt)).unwrap();
    //});
}

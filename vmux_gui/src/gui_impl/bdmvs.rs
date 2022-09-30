use std::path::PathBuf;

use crate::egui;
use crate::gui_common::*;
use bluray_support::TitleInfo;
use egui::*;

use vmux_lib::bd_cache::RGBDsCache;
use vmux_lib::handling::*;

pub struct BDDisplayInfo {
    // bd: bluray_support::BD,
    pub legacy_title_list: Vec<TitleInfo>,

    pub path: String,

    //                  s   path     indexed
    pub strms: Vec<(String, PathBuf, bool)>,
}

impl BDDisplayInfo {
    pub fn new(
        path: &str,
        bdrom: &Bdrom,
        bdbd: &mut RGBDsCache,
        bd_index_dir: &str,
    ) -> Option<BDDisplayInfo> {
        //let bd = bluray_support::BD::open(path);
        let bd = bdbd.get_tis(bdrom);
        let tis = match bd {
            Some(e) => e,
            None => return None,
        };

        let mut strms = Vec::new();
        for (s, _) in bdrom.find_streams() {
            let indexed = bdrom.index_for_stream(&s, bd_index_dir).exists();
            let strm_file = bdrom.find_stream_file(&s);

            strms.push((s, strm_file, indexed));
        }

        Some(BDDisplayInfo {
            //bd,
            legacy_title_list: tis,
            path: path.to_string(),
            strms,
        })
    }
}

fn check_is_addable(id: &str, path: &str, config: &Config) -> bool {
    let criteria_1 = !config.exists_bd(id);
    let criteria_2 = could_be_bdrom_at_path(path);

    let criteria_3 = {
        if id.contains(" ") {
            false
        } else if id == "" {
            false
        } else {
            true
        }
    };
    let criteria_4 = {
        let mut a = true;

        for e in &config.blurays {
            if e.path == path {
                a = false;
                break;
            }
        }

        a
    };

    criteria_1 && criteria_2 && criteria_3 && criteria_4
}

pub fn free_gui_bdmvs(ctx: &egui::Context, asd: &mut GuiGlobalState) {
    let config = &mut asd.vmux_config;

    let mut close_window = false;
    if let Some(e) = asd.bdmvs_addmanager.as_mut() {
        egui::Window::new(format!("Add multi"))
            .collapsible(false)
            .resizable(false)
            // .frame(Frame::window(&ctx.style()).fill(Color32::LIGHT_RED))
            .show(ctx, |ui| {
                if e.len() == 0 {
                    e.push((String::new(), String::new(), false));
                }
                if e[e.len() - 1].2 {
                    e.push((String::new(), String::new(), false));
                }

                ui.heading("Autofind");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut asd.bdmvs_addmanager_tmp_srch);
                    if ui.button("Search").clicked() {
                        let mut to_addd = Vec::new();

                        let pth = PathBuf::from(&asd.bdmvs_addmanager_tmp_srch);
                        for xe in walkdir::WalkDir::new(pth)
                            .into_iter()
                            .filter_map(|e| e.ok())
                        {
                            if xe.file_name() == "00000.m2ts" {
                                let pth = xe.path().parent().unwrap().parent().unwrap();
                                let stro = pth.to_str().unwrap();
                                if !pth.join("PLAYLIST").is_dir() {
                                    continue;
                                }
                                let penn = {
                                    let mut asda = false;
                                    for ent in e.iter() {
                                        if ent.1 == stro {
                                            asda = true;
                                            break;
                                        }
                                    }
                                    asda
                                };
                                if !penn {
                                    to_addd.push({
                                        let a = pth.parent().unwrap();
                                        a.to_str().unwrap().to_string()
                                    });
                                }
                            }
                        }
                        to_addd.sort();
                        for a in to_addd {
                            e.push((String::new(), a, false));
                        }
                    }
                });

                ui.separator();

                if ui.button("Add Empty entry").clicked() {
                    e.push((String::new(), String::new(), false));
                }
                ui.separator();
                ui.heading("Entries to add");
                {
                    let mut config = config.clone();
                    for i in &mut (*e) {
                        ui.horizontal(|ui| {
                            let mut trigger_check = false;

                            if TextEdit::singleline(&mut i.0)
                                .desired_width(200.0)
                                .ui(ui)
                                .changed()
                            {
                                trigger_check = true;
                            }
                            if TextEdit::singleline(&mut i.1)
                                .desired_width(900.0)
                                .ui(ui)
                                .changed()
                            {
                                trigger_check = true;
                            }

                            if trigger_check {
                                i.2 = check_is_addable(&i.0, &i.1, &config);
                            }

                            let btn = if !i.2 {
                                egui::Label::new("bad")
                            } else {
                                egui::Label::new("ok")
                            };
                            btn.ui(ui);
                        });
                        let _ = config.new_bd(i.0.clone(), &i.1).is_ok();
                    }
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("AddAll").clicked() {
                        e.retain(|i| {
                            if i.2 {
                                config.new_bd(i.0.clone(), &i.1).unwrap();
                            }
                            !i.2
                        });
                    }
                    if ui.button("Close").clicked() {
                        close_window = true;
                    }
                });
            });
    }
    if close_window {
        asd.bdmvs_addmanager = None;
    }
}
pub fn gui_bdmvs(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    egui::ScrollArea::vertical()
        .id_source("scroll_gui_bdmvs")
        .show(ui, |ui| {
            if ui.button("MultiAdd").clicked() {
                asd.bdmvs_addmanager = Some(Vec::new());
            }
            ui.collapsing("Add bluray", |ui| {
                ui.label("Path");

                let mut trigger_check = false;
                if ui.text_edit_singleline(&mut asd.tmp_add_path).changed() {
                    trigger_check = true;
                }
                ui.label("InternalId");
                if ui.text_edit_singleline(&mut asd.tmp_internal_id).changed() {
                    trigger_check = true;
                }

                if trigger_check {
                    asd.bdmvs_bd_addable =
                        check_is_addable(&asd.tmp_internal_id, &asd.tmp_add_path, &asd.vmux_config);
                }

                let btn = egui::Button::new("Add bd");
                let btn = if !asd.bdmvs_bd_addable {
                    btn.fill(egui::Color32::RED)
                } else {
                    btn
                };

                if btn.ui(ui).clicked() && asd.bdmvs_bd_addable {
                    let path = asd.tmp_add_path.clone();
                    let id = asd.tmp_internal_id.clone();
                    asd.vmux_config.new_bd(id, &path).unwrap();

                    asd.bdmvs_bd_addable = false;
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Sort").clicked() {
                    asd.vmux_config
                        .blurays
                        .sort_by(|a, b| a.internal_id.partial_cmp(&b.internal_id).unwrap());
                }
                ui.checkbox(&mut asd.bdmvs_longpath, "Longpath");
            });

            let mut delete_this = false;

            ui.collapsing(
                format!(
                    "Bdrom Details ({})",
                    if asd.highlighted_bd.is_some() {
                        "some"
                    } else {
                        "none"
                    }
                ),
                |ui| {
                    if let Some(e) = &mut asd.highlighted_bd {
                        ui.separator();
                        ui.label(format!("name: {}", e.0));
                        {
                            //RENAME
                            if ui
                                .text_edit_singleline(&mut asd.tmp_bdmvs_refacor_rename_src)
                                .changed()
                            {
                                e.1 = true;
                                if asd.tmp_bdmvs_refacor_rename_src == "" {
                                    e.1 = false;
                                }
                                if asd.vmux_config.exists_bd(&asd.tmp_bdmvs_refacor_rename_src) {
                                    e.1 = false;
                                }
                            }
                            let btn = if e.1 {
                                Button::new("RefactorRename").fill(Color32::BLUE)
                            } else {
                                Button::new("RefactorRename").fill(Color32::LIGHT_RED)
                            }
                            .ui(ui);

                            if e.1 {
                                if btn.clicked() {
                                    let old_name = &e.0.clone();
                                    let new_name = &asd.tmp_bdmvs_refacor_rename_src;
                                    for f in &mut asd.vmux_config.folders {
                                        for e in &mut f.entries {
                                            if e.src() != old_name {
                                                continue;
                                            }
                                            match e {
                                            RemuxFolderEntrie::SingularFile(sgrl) => {
                                                sgrl.src = new_name.to_string();
                                            }
                                            RemuxFolderEntrie::MultipleFilePlaylistClipSplit(
                                                mlt,
                                            ) => {
                                                mlt.src = new_name.to_string();
                                            }
                                        }
                                        }
                                    }
                                    e.0 = new_name.to_string();
                                    let new_name = new_name.to_string();
                                    asd.vmux_config.bluray_mut(old_name, |e| {
                                        e.internal_id = new_name;
                                    });
                                    e.1 = false;
                                }
                            }
                        }

                        asd.vmux_config.bluray_mut(&e.0, |ee| {
                            ui.text_edit_singleline(&mut ee.path);
                        });
                        let deb = egui::Button::new("Delete").fill(Color32::RED).ui(ui);
                        if deb.clicked() {
                            delete_this = true;
                        }
                    }
                },
            );
            if delete_this {
                let ee = asd.highlighted_bd.take().unwrap();
                asd.vmux_config.blurays.retain(|e| e.internal_id != ee.0);
                asd.bdsc.clear_for(&ee.0);
            }
            ui.separator();
            if ui.text_edit_singleline(&mut asd.bdmvs_filter).changed() {}
            ui.separator();
            //for f in &asd.vmux_config.blurays {
            for f in &asd.vmux_config.blurays.clone() {
                if asd.bdmvs_filter.len() != 0 {
                    if !f
                        .internal_id
                        .to_owned()
                        .to_lowercase()
                        .contains(&asd.bdmvs_filter.to_owned().to_lowercase())
                    {
                        continue;
                    }
                }

                let selected = if let Some(id) = &asd.highlighted_bd {
                    &f.internal_id == &id.0
                } else {
                    false
                };
                ui.horizontal(|ui| {
                    if asd.bdmvs_longpath {
                        asd.vmux_config.bluray_mut(&f.internal_id, |ee| {
                            TextEdit::singleline(&mut ee.path)
                                .desired_width(f32::INFINITY)
                                .show(ui);
                            //ui.text_edit_singleline(&mut ee.path);
                        });
                    }

                    let mut inspect_btn = Button::new("Inspect");
                    if let Some(e) = asd.inspect_bd.as_ref() {
                        if e.0 == f.internal_id {
                            inspect_btn = inspect_btn.fill(Color32::BLUE);
                        }
                    }

                    if inspect_btn.ui(ui).clicked() {
                        let disp = BDDisplayInfo::new(
                            &f.path,
                            f,
                            &mut asd.bdsc,
                            &asd.vmux_config.bd_index_dir,
                        );
                        match disp {
                            Some(e) => {
                                asd.inspect_bd = Some((f.internal_id.clone(), e));
                            }
                            None => {
                                asd.throw_error(format!("Could not open bluray {}", f.internal_id))
                            }
                        }
                    }

                    if ui
                        .selectable_label(selected, format!("{}", f.internal_id))
                        .clicked()
                    {
                        let flag = if let Some(hel) = &asd.highlighted_bd {
                            if hel.0 == f.internal_id {
                                false
                            } else {
                                true
                            }
                        } else {
                            true
                        };
                        if flag {
                            asd.highlighted_bd = Some((f.internal_id.clone(), false));
                        } else {
                            asd.highlighted_bd = None;
                        }
                    }
                });
            }
        });
}

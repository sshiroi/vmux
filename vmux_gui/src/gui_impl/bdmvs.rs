use crate::egui;
use crate::gui_common::*;
use egui::*;

use vmux_lib::handling::*;

pub fn gui_bdmvs(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    egui::ScrollArea::vertical()
        .id_source("scroll_gui_bdmvs")
        .show(ui, |ui| {
            ui.collapsing("Add bluray", |ui| {
                ui.label("Path");
                if ui.text_edit_singleline(&mut asd.tmp_add_path).changed() {
                    asd.tmp_add_is_bd_addable1 = could_be_bdrom_at_path(&asd.tmp_add_path);
                }
                ui.label("InternalId");
                if ui.text_edit_singleline(&mut asd.tmp_internal_id).changed() {
                    asd.tmp_add_is_bd_addable2 = !asd.vmux_config.exists_bd(&asd.tmp_internal_id);
                }

                let nott = !asd.tmp_add_is_bd_addable1 || !asd.tmp_add_is_bd_addable2;

                let btn = egui::Button::new("Add bd");
                let btn = if nott {
                    btn.fill(egui::Color32::RED)
                } else {
                    btn
                };

                if btn.ui(ui).clicked() && !nott {
                    let path = asd.tmp_add_path.clone();
                    let id = asd.tmp_internal_id.clone();
                    asd.vmux_config.new_bd(id, &path).unwrap();

                    asd.tmp_add_is_bd_addable2 = false;
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

                    if ui.button("Inspect").clicked() {
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

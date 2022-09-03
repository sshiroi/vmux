use eframe::egui::{PointerButton, Widget};
use eframe::epaint::Color32;

use crate::egui;
use crate::gui_common::GuiGlobalState;

pub fn gui_bd_folders(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    egui::ScrollArea::vertical()
        .id_source("bd_folderss")
        .show(ui, |ui| {
            ui.collapsing("Add Folder", |ui| {
                ui.label("Name");
                if ui
                    .text_edit_singleline(&mut asd.tmp_add_folder_path)
                    .changed()
                {}
                if ui.button("Add folder").clicked() {
                    let path = asd.tmp_add_folder_path.clone();
                    match asd.vmux_config.new_folder(&path, true) {
                        Ok(_) => {}
                        Err(_) => println!("Folder already exists"),
                    };
                }
            });

            ui.separator();
            let mut delete = None;
            ui.collapsing("Details", |ui| {
                if ui.selectable_label(false, "Deselect").clicked() {
                    asd.selected_folder = None;
                    asd.folders_selection_idx = None;
                }
                if let Some(e) = &asd.selected_folder {
                    let fld = &mut asd.vmux_config.folders[*e];
                    if ui.button("Export").clicked() {
                        asd.folder_export = Some(fld.clone());
                    }

                    ui.label("Name:");
                    if ui.text_edit_singleline(&mut fld.name).changed() {}
                    ui.label("File Prefix:");
                    if ui.text_edit_singleline(&mut fld.file_prefix).changed() {}

                    let deb = egui::Button::new("Delete").fill(Color32::RED).ui(ui);
                    if deb.clicked() {
                        delete = Some(*e);
                    }
                }
            });
            ui.separator();

            if ui.text_edit_singleline(&mut asd.folders_filter).changed() {
                asd.folders_selection_idx = None;
            }

            ui.separator();

            ui.heading("Folders");
            ui.label("show:fullload");

            let mut mass_switch = None;

            let mut real_i = 0;
            for (i, f) in asd.vmux_config.folders.iter_mut().enumerate() {
                if asd.folders_filter.len() != 0 {
                    if !f
                        .name
                        .to_owned()
                        .to_lowercase()
                        .contains(&asd.folders_filter.to_owned().to_lowercase())
                    {
                        continue;
                    }
                }

                let selected = if let Some(id) = &asd.selected_folder {
                    &i == id
                } else {
                    false
                };

                ui.horizontal(|ui| {
                    if let Some(e) = asd.folders_selection_idx {
                        if real_i == e.0 {
                            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::LIGHT_YELLOW;
                        } else {
                            ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::DARK_BLUE;
                        }
                    }
                    if ui
                        .checkbox(&mut f.show, "")
                        .clicked_by(PointerButton::Secondary)
                    {
                        if asd.folders_selection_idx.is_none() {
                            asd.folders_selection_idx = Some((real_i, !f.show));
                        } else {
                            mass_switch = Some(real_i as usize);
                        }
                    }
                    if asd.folders_selection_idx.is_some() {
                        ui.reset_style();
                    }
                    ui.checkbox(&mut f.full_load, "");
                    let mut do_select = false;
                    if ui
                        .selectable_label(selected, format!("{}", f.name))
                        .context_menu(|ui| {
                            let _ = ui.text_edit_singleline(&mut f.name);

                            /*
                             if ui.button("Select").clicked() {
                                 do_select = true;
                                 ui.close_menu();
                             }
                             if ui.button("Delete").clicked() {
                                delete = Some(i);
                                ui.close_menu();
                            }
                            */
                        })
                        .clicked()
                    {
                        do_select = true;
                    }
                    if do_select {
                        asd.selected_folder = Some(i);
                        asd.remux_folder_flattend = None;
                        asd.folders_selection_idx = None;
                        asd.remux_folder_last_hash_all = 0;
                        asd.remux_folder_lasthash = 0;
                    }
                });
                real_i += 1;
            }
            if let Some(e) = &asd.folders_selection_idx {
                if let Some(e2) = &mass_switch {
                    let min = (*e2).min(e.0);
                    let max = (*e2).max(e.0);
                    for (i, f) in asd.vmux_config.folders.iter_mut().enumerate() {
                        if asd.folders_filter.len() != 0 {
                            if !f
                                .name
                                .to_owned()
                                .to_lowercase()
                                .contains(&asd.folders_filter.to_owned().to_lowercase())
                            {
                                continue;
                            }
                        }
                        if i >= min && i <= max {
                            f.show = e.1;
                        }
                    }
                }
            }
            if asd.folders_selection_idx.is_some() && mass_switch.is_some() {
                asd.folders_selection_idx = None;
            }

            if let Some(d) = delete {
                asd.selected_folder = None;
                asd.vmux_config.folders.remove(d);
            }
        });
}

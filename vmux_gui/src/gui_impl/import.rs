use crate::gui_common::GuiGlobalState;
use eframe::egui::{self, Widget};
use vmux_lib::handling::Exporter;

pub fn gui_import(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    egui::ScrollArea::vertical()
        .id_source("asdasdasd_import")
        .show(ui, |ui| {
            if ui.text_edit_singleline(&mut asd.import_textbox).changed() {
                //if ui.text_edit_multiline(&mut asd.import_textbox).changed() {
                let txt = asd.import_textbox.clone();
                //let new = txt;
                let new = txt.replace("\n", "").to_owned();
                let new = new.trim();

                let epx = Exporter::from_string(&new);
                if epx.is_some() {
                    asd.import_error = None;
                } else {
                    asd.import_error = Some("did not import".to_owned());
                }
                if let Some(e) = epx {
                    asd.import_result = Some((
                        e.blurays
                            .into_iter()
                            .map(|e| (String::new(), false, e))
                            .collect(),
                        e.folders,
                    ));
                } else {
                    asd.import_result = None;
                }
            }
            if ui.button("Clear").clicked() {
                asd.import_error = None;
                asd.import_result = None;
                asd.import_textbox = "".to_string();
            }
            if let Some(e) = &mut asd.import_result {
                ui.heading("Bdroms");

                for bd in &mut e.0 {
                    ui.horizontal(|ui| {
                        if ui.text_edit_singleline(&mut bd.0).changed() {
                            bd.1 = vmux_lib::config::could_be_bdrom_at_path(&bd.0);
                        }

                        let all_ok = bd.1 && !asd.vmux_config.exists_bd(&bd.2.internal_id);

                        let btn = egui::Button::new(format!("Import {}", &bd.2.internal_id));
                        let btn = if asd.vmux_config.exists_bd(&bd.2.internal_id) {
                            btn.fill(egui::Color32::GOLD)
                        } else if all_ok {
                            btn.fill(egui::Color32::GREEN)
                        } else {
                            btn.fill(egui::Color32::RED)
                        };
                        if btn.ui(ui).clicked() {
                            if all_ok {
                                asd.bdsc.clear_for(&bd.2.internal_id);
                                bd.2.path = bd.0.clone();

                                asd.vmux_config.blurays.push(bd.2.clone());
                            }
                        }
                    });
                }
                ui.heading("Folders");

                let import_all = if e.1.len() > 1 {
                    ui.button("Try Import all").clicked()
                } else {
                    false
                };
                for fld in &e.1 {
                    let btn = egui::Button::new(format!("Import {}", &fld.name));
                    let all_ok = !asd.vmux_config.exists_folder(&fld.name);
                    let btn = if !all_ok {
                        btn.fill(egui::Color32::GOLD)
                    } else {
                        btn.fill(egui::Color32::GREEN)
                    };
                    if import_all || btn.ui(ui).clicked() {
                        if all_ok {
                            asd.vmux_config.folders.push(fld.clone());
                        }
                    }

                    egui::CollapsingHeader::new(&fld.name).show(ui, |ui| {
                        for e in &fld.entries {
                            ui.label(format!("{} - {}", e.src(), e.name()));
                        }
                    });
                    ui.separator();
                }
            }

            ui.separator();
        });
}

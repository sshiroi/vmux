use crate::egui;
use crate::gui_common::*;

use super::{GuiIndexQueue, IndexQueuEntry};

pub fn gui_clips(ui: &mut egui::Ui, asd: &mut GuiGlobalState, index_queue: &mut GuiIndexQueue) {
    egui::ScrollArea::vertical()
        .id_source("scroll_gui_clipszz")
        .show(ui, |ui| {
            if let Some((romid, bdi)) = &mut asd.inspect_bd {
                let idx_all = ui.button("IndexAll").clicked();

                for (s, strm_file, indexed) in &mut bdi.strms {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", s, indexed));

                        if !*indexed {
                            if ui.button("Index").clicked() || idx_all {
                                index_queue
                                    .index_queue
                                    .push(IndexQueuEntry::new(romid, s.to_owned()));

                                super::index_queue::check_trigger_indexing(
                                    index_queue,
                                    &asd.vmux_config,
                                );
                                //not necesserly the case but hides the button
                                *indexed = true;
                            }
                        }

                        if ui.button("Play").clicked() {
                            let ss = strm_file.to_str().unwrap().to_owned();
                            std::thread::spawn(move || {
                                std::process::Command::new("mpv")
                                    .args([format!("{}", ss)])
                                    .output()
                                    .expect("failed to execute process");
                            });
                        }
                    });
                }
            }
            ui.separator();
            ui.heading("Index Queue");
            super::gui_index_queue(ui, index_queue, &asd.vmux_config);
        });
}

use eframe::egui;
use vmux_lib::handling::RemuxFolderEntrie;

use crate::gui_common::GuiGlobalState;

pub fn free_export_frame(ctx: &egui::Context, glob: &mut GuiGlobalState) {
    //TODO: move into free_ function
    if glob.folder_export.is_some()
        || (glob.folder_export_other && glob.folder_export_string.is_some())
    {
        egui::Window::new(format!("Export?"))
            .collapsible(false)
            .resizable(false)
            // .frame(Frame::window(&ctx.style()).fill(Color32::LIGHT_RED))
            .show(ctx, |ui| {
                if !glob.folder_export_other {
                    let name = glob.folder_export.as_ref().unwrap().name.clone();

                    ui.heading(format!("Export - {}", name));
                } else {
                    ui.heading(format!("Export - Other"));
                }

                //ui.checkbox(&mut glob.folder_export_yaml, "yaml");
                glob.folder_export_yaml = false;

                if ui.button("Close").clicked() {
                    glob.folder_export = None;
                    glob.folder_export_string = None;
                    glob.folder_export_other = false;
                    return;
                }

                if let Some(e) = &glob.folder_export_string {
                    ui.text_edit_multiline(&mut e.clone());
                } else {
                    let name = glob.folder_export.as_ref().unwrap().name.clone();

                    let e = glob.folder_export.as_ref().unwrap();

                    ui.horizontal(|ui| {
                        let mut exprt_some = None;
                        if ui.button("With Bdroms").clicked() {
                            let mut exp = vmux_lib::config::Exporter::new();
                            exp.add_folder(e);
                            let strc = collect_bdrom_src(&e.entries);

                            for e in &strc {
                                if let Some(bdr) = glob.vmux_config.bluray(e) {
                                    exp.add_bdrom(bdr);
                                }
                            }
                            exprt_some = Some((format!("[{}_bdroms]", name), exp));
                        }
                        if ui.button("Without Bdroms").clicked() {
                            let mut exp = vmux_lib::config::Exporter::new();
                            exp.add_folder(e);
                            exprt_some = Some((format!("[{}_nobdroms]", name), exp));
                        }
                        if ui.button("Bdroms only").clicked() {
                            let mut exp = vmux_lib::config::Exporter::new();
                            let strc = collect_bdrom_src(&e.entries);

                            for e in &strc {
                                if let Some(bdr) = glob.vmux_config.bluray(e) {
                                    exp.add_bdrom(bdr);
                                }
                            }
                            exprt_some = Some((format!("[{}_only_bdroms]", name), exp));
                        }
                        if let Some(e) = exprt_some {
                            if !glob.folder_export_yaml {
                                glob.folder_export_string =
                                    Some(format!("{}{}", e.0, e.1.string_out()));
                            } else {
                                glob.folder_export_string = Some(e.1.string_out_txt_uncompressed());
                            }
                        }
                    });
                }
            });
    }
}

fn collect_bdrom_src(es: &[RemuxFolderEntrie]) -> Vec<String> {
    let mut strc: Vec<String> = Vec::new();
    for e in es {
        let aa = e.src().to_owned();
        if !strc.contains(&aa) {
            strc.push(aa);
        }
    }
    strc
}

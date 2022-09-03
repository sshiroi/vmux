use crate::egui;
use crate::gui_common::*;

pub fn gui_config(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    ui.horizontal(|ui| {
        ui.label("IndexDir: ");
        ui.text_edit_singleline(&mut asd.vmux_config.bd_index_dir);
    });

    ui.horizontal(|ui| {
        ui.label("EBL Save location");
        ui.text_edit_singleline(&mut asd.vmux_config.mpvraw_exportlocation);
    });
    if ui.button("Save config&cache").clicked() {
        asd.vmux_config.save();
        asd.bdsc.save();
        asd.ondisk_vmux_config = asd.vmux_config.clone();
    }
    if ui.button("Export all").clicked() {
        let mut exp = vmux_lib::config::Exporter::new();

        exp.add_folders(&asd.vmux_config.folders);
        exp.add_bdroms(&asd.vmux_config.blurays);

        asd.folder_export_string = Some(format!("[all_exporteverything]{}", exp.string_out()));
        asd.folder_export_other = true;
    }
    ui.separator();
    if ui.button("Clear cache").clicked() {
        asd.bdsc.clear_disk();
    }
    ui.separator();
    let mut infos = Vec::new();
    for f in &asd.vmux_config.blurays {
        let ondisk = asd.ondisk_vmux_config.bluray(&f.internal_id);
        match ondisk {
            Some(e) => {
                if e != f {
                    infos.push(format!("Changed {}", f.internal_id));
                    if e.path != f.path {
                        infos.push(format!(" path: {}", f.path));
                    }
                    if e.title_comments != f.title_comments {
                        infos.push(format!(" comments: changed"));
                    }
                }
            }
            None => infos.push(format!("Added {}", f.internal_id)),
        }
    }
    for f in &asd.ondisk_vmux_config.blurays {
        let inram = asd.vmux_config.bluray(&f.internal_id);
        if inram.is_none() {
            infos.push(format!("Removed {}", f.internal_id));
        }
    }

    if asd.ondisk_vmux_config == asd.vmux_config {
        ui.label("There are no changes");
    } else {
        ui.label("There are unsaved changes");
    }
    ui.separator();
    ui.heading("Changelog");
    if infos.len() == 0 {
        ui.label(format!("No changelog"));
    } else {
        for l in infos {
            ui.label(l);
        }
    }
}

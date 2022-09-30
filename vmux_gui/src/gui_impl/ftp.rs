use vmux_lib::fs::HelloFsEntry;

use crate::egui;
use crate::gui_common::*;

pub(crate) fn build_filelist_from_fls(fls: &[HelloFsEntry]) -> Vec<String> {
    let mut all = collect_all_abs(fls, "".to_string());
    all.sort();

    all
}

pub(crate) fn collect_all_abs(a: &[HelloFsEntry], current: String) -> Vec<String> {
    let mut asd = Vec::new();
    for e in a {
        match e {
            HelloFsEntry::HelloFile(f) => asd.push(format!("{}/{}", current, f.name)),
            HelloFsEntry::HelloFolder(fol) => {
                if current == "" {
                    asd.append(&mut collect_all_abs(&fol.inner, format!("{}", fol.name)));
                } else {
                    asd.append(&mut collect_all_abs(
                        &fol.inner,
                        format!("{}/{}", current, fol.name),
                    ));
                }
            }
        };
    }
    asd
}

fn put_in_collaps(port: u16, last: &str, ui: &mut egui::Ui, lst: &[String]) {
    ui.collapsing(last, |ui| {
        egui::Grid::new(format!("my_grid22_{}", last))
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for fil in lst {
                    let splt: Vec<&str> = fil.split("/").collect();

                    ui.label(splt[1]);
                    if ui.button("Play").clicked() {
                        let ss = fil.clone();
                        std::thread::spawn(move || {
                            std::process::Command::new("mpv")
                                .args([format!("ftp://127.0.0.1:{}/{}", port, ss)])
                                .output()
                                .expect("failed to execute process");
                        });
                    }
                    ui.end_row();
                }
            });
    });
}

pub fn gui_ftp(ui: &mut egui::Ui, asd: &mut GuiGlobalState) {
    egui::ScrollArea::vertical()
        .id_source("ftppp")
        .show(ui, |ui| {
            ui.label(format!("status: {}", asd.ftp.is_some()));
            ui.separator();

            let mut remove = false;
            if let Some(e) = &asd.ftp {
                ui.label(format!("Port: {}", e.0));
                if ui.button("Stop").clicked() {
                    let asd = e.1.clone();
                    *(asd.lock().unwrap()) = true;
                    remove = true;
                }

                if ui.text_edit_singleline(&mut asd.ftp_search).clicked() {}

                let mut collected_lst = Vec::new();
                let mut last = String::new();
                let mut first = true;
                for fil in &e.2 {
                    if asd.ftp_search != "" {
                        if !fil
                            .to_owned()
                            .to_lowercase()
                            .contains(&(asd.ftp_search.to_owned().to_lowercase()))
                        {
                            continue;
                        }
                    }
                    let splt: Vec<&str> = fil.split("/").collect();
                    if first {
                        last = splt[0].to_string();
                        first = false;
                    }
                    if splt[0] == last {
                        collected_lst.push(fil.to_string());
                    } else {
                        put_in_collaps(e.0, &last, ui, &collected_lst);
                        collected_lst.clear();
                        last = splt[0].to_string();
                        collected_lst.push(fil.to_string());
                    }
                }
                if !first {
                    put_in_collaps(e.0, &last, ui, &collected_lst);
                }
            } else {
                number_widge(ui, &mut asd.vmux_config.ftp_port, "port");
                if ui.button("Start").clicked() {
                    let mut avbd = vmux_lib::bd_cache::AVBDsCache::new();
                    let fls = vmux_lib::ftp::build_fls(&mut avbd, &asd.vmux_config);
                    let flllll = build_filelist_from_fls(&fls);
                    let stopy = vmux_lib::ftp::spawn_combined(
                        asd.vmux_config.clone(),
                        asd.vmux_config.ftp_port,
                        fls,
                    );
                    asd.ftp = Some((asd.vmux_config.ftp_port, stopy, flllll));
                }
            }
            if remove {
                asd.ftp = None;
            }
        });
}

fn number_widge<'a>(ui: &mut egui::Ui, trgt: &'a mut u16, prfx: &str) {
    //    let scroll_speed = 0.007;
    //    ui.add(egui::DragValue::new(trgt).prefix(prfx).speed(scroll_speed));
    ui.horizontal(|ui| {
        ui.label(prfx);
        let mut txt = format!("{}", trgt);
        ui.text_edit_singleline(&mut txt);
        *trgt = txt.parse::<u16>().ok().or(Some(*trgt)).unwrap();
    });
}

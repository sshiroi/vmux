use eframe::egui;
use vmux_lib::handling::PlaylistId;

use crate::gui_common::GuiGlobalState;

#[derive(Default)]
pub struct VlcOutput {
    pub inspect_bdromid: String,
    pub raw_lines: Vec<String>,

    pub found_playlist: Vec<(u64, String,bool)>,
}

fn contains_key(a: &[(u64, String,bool)], key: u64) -> bool {
    for a in a {
        if a.0 == key {
            return true;
        }
    }
    false
}

pub fn vlc_process_line(output: &mut VlcOutput, a: &str) {
    let srch = "HDMV_EVENT_PLAY_PL(4): ";
    let pl = a.find(srch);
    if let Some(e) = pl {
        let num = &a[e + srch.len()..];

        let asd_number: Result<u64, _> = num.parse();
        if let Ok(num) = asd_number {
    //        if !contains_key(&output.found_playlist, num) {
                output.found_playlist.push((num, String::new(),false));
     //       }
        }
    }
}
pub fn vlc_ui(ui: &mut egui::Ui, output: &mut VlcOutput, asd: &mut GuiGlobalState) {
    ui.heading("Collected playlist");
    ui.checkbox(&mut asd.vlc_filter_already, "Filter already");
    let bd = asd.vmux_config.bluray(&output.inspect_bdromid);
    if bd.is_none() {
        return;
    }
    let bd = bd.unwrap();
    if asd.vlc_filter_already {
        for e in &mut output.found_playlist {
            let asd = bd.getcreate_title_comment(PlaylistId::from_pis(e.0));
            if asd.name != "" {
                e.1 = asd.name;
                e.2 = true;
            }
        }
        
    }

    let mut remove = None;
    let ssss_len = output.found_playlist.len();
    for (ii, pl) in output.found_playlist.iter_mut().rev().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("PL: {}", pl.0));
            if pl.2 {
                ui.label(format!("named: {}",&pl.1));
            }else {
            ui.text_edit_singleline(&mut pl.1);
            if ui.button("Name").clicked() {
                asd.vmux_config.bluray_mut(&output.inspect_bdromid, |bd| {
                    let pid = PlaylistId::from_pis(pl.0);
                    let mut asd = bd.getcreate_title_comment(pid);
                    asd.name = pl.1.to_owned();
                    bd.set_title_comment(pid, asd);
                });
            }
        }
            if ui.button("Remove").clicked() {
                remove = Some(ssss_len - 1 - ii);
            }
        });
    }
    if let Some(ii) = remove {
        output.found_playlist.remove(ii);
    }
    ui.separator();
    ui.heading("Vlc output");

    if ui.button("Clear").clicked() {
        output.raw_lines.clear();
    }
    for ln in output.raw_lines.iter().rev() {
        ui.label(ln);
    }
}

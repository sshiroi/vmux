use eframe::egui::{self, TextEdit};
use vmux_lib::handling::{PlaylistId, RemuxFolderEntrie, SingularRemuxMatroskaFile};

use crate::gui_common::GuiGlobalState;

#[derive(Debug)]
pub enum ChapterAttention {
    None,
    One(u64),
    Two(u64, u64),
}

impl Default for ChapterAttention {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub struct VlcOutput {
    pub inspect_bdromid: String,
    pub raw_lines: Vec<String>,

    pub found_playlist: Vec<(u64, String, bool)>,

    pub last_title: u64,

    pub pl_chapter_attention: u64,
    pub chapter_attention: ChapterAttention,
    pub auto_add: bool,
}

#[allow(unused)]
fn contains_key(a: &[(u64, String, bool)], key: u64) -> bool {
    for a in a {
        if a.0 == key {
            return true;
        }
    }
    false
}

pub fn vlc_process_line(output: &mut VlcOutput, a: &str) {
    let srch1 = "HDMV_EVENT_PLAY_PL(4): ";
    let playlist_event = a.find(srch1);
    let srch2 = "HDMV_EVENT_PLAY_PM(6): ";
    let chapter_event = a.find(srch2);
    if let Some(e) = playlist_event {
        let num = &a[e + srch1.len()..];
        let asd_number: Result<u64, _> = num.parse();

        if let Ok(num) = asd_number {
            output.found_playlist.push((num, String::new(), false));
            output.last_title = num;
        }
    }
    if let Some(e) = chapter_event {
        let num = &a[e + srch2.len()..];
        let asd_number: Result<u64, _> = num.parse();

        if let Ok(num) = asd_number {
            if output.pl_chapter_attention == output.last_title {
                output.chapter_attention = match output.chapter_attention {
                    ChapterAttention::None => ChapterAttention::One(num),
                    ChapterAttention::One(a) => ChapterAttention::Two(a, num),
                    ChapterAttention::Two(_, _) => ChapterAttention::One(num),
                };
            }
        }
    }
}
pub fn vlc_ui(ui: &mut egui::Ui, output: &mut VlcOutput, asd: &mut GuiGlobalState) {
    egui::ScrollArea::vertical()
        .id_source("vlc_uiUU")
        .show(ui, |ui| {
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
                        ui.label(format!("named: {}", &pl.1));
                    } else {
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
            ui.heading("Episode Chapter helper");
            ui.horizontal(|ui| {
                if ui.button("Reset").clicked() {
                    output.chapter_attention = Default::default();
                }
                let mut text = format!("{}", output.pl_chapter_attention);
                if TextEdit::singleline(&mut text)
                    .desired_width(30.0)
                    .show(ui)
                    .response
                    .changed()
                {
                    let nzum: Result<u64, _> = text.parse();
                    if let Ok(e) = nzum {
                        output.pl_chapter_attention = e;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut output.auto_add, "autoadd");
                ui.label(format!("{:?}", &output.chapter_attention));
                let mut added = false;
                if let ChapterAttention::Two(a, b) = output.chapter_attention {
                    if let Some(ii) = &asd.selected_folder {
                        if ui.button("Add").clicked() || output.auto_add {
                            let fld = &mut asd.vmux_config.folders[*ii];
                            //TODO: add singular chapter
                            fld.entries.push(RemuxFolderEntrie::SingularFile(
                                SingularRemuxMatroskaFile::new(
                                    format!("{} {}", a, b),
                                    output.inspect_bdromid.to_string(),
                                    vmux_lib::handling::BlurayExtract::PlaylistFromToChap(
                                        PlaylistId::from_pis(output.pl_chapter_attention),
                                        a as u64,
                                        b as u64,
                                    ),
                                ),
                            ));
                            added = true;
                        }
                    }
                }
                if added {
                    output.chapter_attention = ChapterAttention::None;
                }
            });
            ui.separator();

            ui.heading("Vlc output");

            if ui.button("Clear").clicked() {
                output.raw_lines.clear();
            }
            for ln in output.raw_lines.iter().rev() {
                ui.label(ln);
            }
        });
}

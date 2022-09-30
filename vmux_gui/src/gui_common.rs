use crate::egui;
use crate::egui_docking;
use crate::gui_impl::free_gui_bdmvs;
use crate::gui_impl::vlc_ui;
use egui::{Color32, Frame};
use vmux_lib::handling::*;

use crate::gui_impl;

//TODO: remove
pub use crate::global_state::GuiGlobalState;

pub struct EguiAppState {
    pub global: GuiGlobalState,
    pub index_queue: gui_impl::GuiIndexQueue,
    // pub errors: gui_impl::GuiErrorsState,
}

pub struct EguiApp {
    pub state: EguiAppState,
    pub egtre: egui_docking::Tree<EguiAppState>,
}

impl EguiApp {
    pub fn new(cfg: Config) -> Self {
        use egui_docking::NodeIndex;

        let mut egtre = egui_docking::Tree::new(vec![
            Box::new(PlaceholderTab::Inspect),
            Box::new(PlaceholderTab::Ftp),
            Box::new(PlaceholderTab::Clips),
            Box::new(PlaceholderTab::Import),
            Box::new(PlaceholderTab::Config),
        ]);

        //Default layout splits
        let [a, b] = egtre.split_left(
            NodeIndex::root(),
            0.16,
            vec![Box::new(PlaceholderTab::Bdvms)],
        );
        egtre.split_below(b, 0.5, vec![Box::new(PlaceholderTab::Folders)]);
        let [c, _] = egtre.split_right(a, 0.5, vec![Box::new(PlaceholderTab::RemuxFolders)]);
        egtre.split_below(c, 0.5, vec![Box::new(PlaceholderTab::Vlc)]);
        Self {
            state: EguiAppState::new(cfg),
            egtre,
        }
    }
}

impl EguiAppState {
    fn new(cfg: Config) -> Self {
        Self {
            global: GuiGlobalState::new(cfg),
            index_queue: Default::default(),
            //      errors: Default::default(),
        }
    }
}

impl eframe::App for EguiApp {
    fn on_exit_event(&mut self) -> bool {
        let glob = &mut self.state.global;
        if glob.vmux_config != glob.ondisk_vmux_config {
            glob.is_exiting = true;
            glob.can_exit
        } else {
            true
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        {
            let glob = &mut self.state.global;

            if glob.is_exiting {
                if gui_impl::show_exit_window(&ctx, glob) {
                    frame.quit();
                }
            }

            let mut cntrr = 0;
            glob.current_msg_errors.retain(|e| {
                cntrr += 1;
                let mut asd = true;
                egui::Window::new(format!("Error? {}", e.0))
                    .collapsible(false)
                    .resizable(false)
                    .frame(Frame::window(&ctx.style()).fill(Color32::LIGHT_RED))
                    .show(ctx, |ui| {
                        ui.heading(&e.1);
                        if ui.button("Close").clicked() {
                            asd = false;
                        }
                    });
                asd
            });

            free_gui_bdmvs(&ctx, glob);

            gui_impl::free_export_frame(&ctx, glob);
        }

        let style = egui_docking::Style::from_egui(ctx.style().as_ref());

        let id = egui::Id::new("some hashable string");
        let layer_id = egui::LayerId::background();
        let max_rect = ctx.available_rect();
        let clip_rect = ctx.available_rect();

        let mut ui = egui::Ui::new(ctx.clone(), layer_id, id, max_rect, clip_rect);
        egui_docking::show(&mut ui, id, &style, &mut self.egtre, &mut self.state);

        ctx.request_repaint();
    }
}

pub enum PlaceholderTab {
    // Errors,
    Ftp,
    //   Index,
    Clips,
    Bdvms,
    Folders,
    RemuxFolders,
    Config,
    Vlc,
    Inspect,
    Import,
}

impl egui_docking::Tab<EguiAppState> for PlaceholderTab {
    fn title(&self) -> &str {
        match self {
            //PlaceholderTab::Errors => "Errors",
            //        PlaceholderTab::Index => "Index",
            PlaceholderTab::Ftp => "Ftp",
            PlaceholderTab::Clips => "BdStreams", //TODO: also change clips everywhere to BdStreams
            PlaceholderTab::Bdvms => "Bdmvs",
            PlaceholderTab::Folders => "Folders",
            PlaceholderTab::RemuxFolders => "RemuxFolders",
            PlaceholderTab::Config => "Config",
            PlaceholderTab::Inspect => "Inspect",
            PlaceholderTab::Vlc => "Vlc",
            PlaceholderTab::Import => "Import",
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EguiAppState) {
        let margin = egui::style::Margin::same(4.0);

        egui::Frame::none()
            .inner_margin(margin)
            .show(ui, |ui| match self {
                //    PlaceholderTab::Errors => {
                //        crate::gui_impl::gui_check_errors(ui, &mut ctx.errors, &ctx.global.vmux_config)
                //    }

                //   PlaceholderTab::Index => {},
                PlaceholderTab::Clips => {
                    crate::gui_impl::gui_clips(ui, &mut ctx.global, &mut ctx.index_queue)
                }

                PlaceholderTab::Bdvms => crate::gui_impl::gui_bdmvs(ui, &mut ctx.global),

                PlaceholderTab::Ftp => crate::gui_impl::gui_ftp(ui, &mut ctx.global),

                PlaceholderTab::Folders => crate::gui_impl::gui_bd_folders(ui, &mut ctx.global),

                PlaceholderTab::RemuxFolders => {
                    crate::gui_impl::gui_remux_folder(ui, &mut ctx.global)
                }

                PlaceholderTab::Config => crate::gui_impl::gui_config(ui, &mut ctx.global),
                PlaceholderTab::Inspect => crate::gui_impl::gui_bd_inspection(ui, &mut ctx.global),
                PlaceholderTab::Import => crate::gui_impl::gui_import(ui, &mut ctx.global),
                PlaceholderTab::Vlc => {
                    let ll = ctx.global.vlc_output.clone();
                    let mut ls = ll.lock().unwrap();
                    if ls.is_some() {
                        vlc_ui(ui, ls.as_mut().unwrap(), &mut ctx.global);
                    }
                }
            });
    }
}

use eframe::{
    egui::{self, Widget},
    epaint::Color32,
};

use crate::gui_common::GuiGlobalState;

pub fn show_exit_window(ctx: &egui::Context, glob: &mut GuiGlobalState) -> bool {
    let mut do_quit = false;
    egui::Window::new("quitw?")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Do you want to quit? There are changes that aren't saved.");
            ui.horizontal(|ui| {
                let close = egui::Button::new("Close").fill(Color32::RED).ui(ui);
                let cancel = egui::Button::new("Cancel").ui(ui);
                let save = egui::Button::new("Save").fill(Color32::GREEN).ui(ui);
                if close.clicked() {
                    glob.can_exit = true;
                    do_quit = true;
                }
                if cancel.clicked() {
                    glob.is_exiting = false;
                }

                if save.clicked() {
                    //Config
                    glob.vmux_config.save();
                    //Cache
                    glob.bdsc.save();

                    glob.can_exit = true;
                    do_quit = true;
                }
            });
        });
    do_quit
}

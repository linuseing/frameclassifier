use rfd::FileDialog;

use crate::{app::{App, View}, utils::load_video};

pub fn render(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Home");

        ui.horizontal(|ui| {
            if ui.button("Open Video").clicked() {
                if let Some(path) = FileDialog::new().add_filter("Video", &["mp4", "avi", "mov"]).pick_file() {
                    app.video_path = Some(path);
                }
            }

            if ui.button("Label").clicked() {
                if let Some(path) = app.video_path.as_ref() {
                    app.video_capture = Some(load_video(path));
                    app.is_playing = false;
                    app.current_view = View::Label;
                }
            }
        });
    });
}
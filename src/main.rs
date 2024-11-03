mod app;
mod project;
mod utils;
mod views;

fn main() {
    let app = app::App::new();
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Frame Classifier",
        options,
        Box::new(move |_cc| Ok(Box::new(app))),
    );
}

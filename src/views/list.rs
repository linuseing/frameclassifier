use std::{fs, path::PathBuf, process::exit};

use crate::{
    app::{FrameAnnotation, GlobalState},
    utils::load_video,
};
use eframe::egui::{self, ProgressBar};
use opencv::{
    imgcodecs,
    videoio::{self, VideoCaptureTrait},
};

use super::{home::HomeView, View};

pub struct ListView {}

impl View for ListView {
    fn render(&mut self, ctx: &egui::Context, app: &mut GlobalState) -> Option<Box<dyn View>> {
        let mut next_view = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            next_view = list_ui(ui, app);
            label_list(ui, app);
            if app.show_export_popup {
                export_labels_popup(ui, app, ctx);
            }
        });
        next_view
    }
}

impl ListView {
    pub fn new() -> Self {
        Self {}
    }
}

fn list_ui(ui: &mut egui::Ui, app: &mut GlobalState) -> Option<Box<dyn View>> {
    let mut next_view = None;
    ui.horizontal(|ui| {
        if ui.button("Home").clicked() {
            next_view = Some(Box::new(HomeView::new()) as Box<dyn View>);
        }
        if ui.button("Export").clicked() {
            app.export_progress = Some(export_labels(
                &app.video_path.clone().unwrap(),
                app.annotations.clone(),
            ));
            app.show_export_popup = true;
        }
    });
    next_view
}

fn label_list(ui: &mut egui::Ui, app: &mut GlobalState) -> Option<Box<dyn View>> {
    let mut to_delete = Vec::new();
    for (video, annotations) in app
        .project
        .as_mut()
        .unwrap()
        .annotations
        .iter()
        .filter(|(_, annotations)| !annotations.is_empty())
    {
        ui.label(&format!("{}", video));
        for (i, annotation) in annotations.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(&format!(
                    "{}: {} - {}",
                    annotation.label, annotation.start_frame, annotation.end_frame
                ));
                if ui.button("Delete").clicked() {
                    to_delete.push((video.clone(), i));
                }
            });
        }
    }
    for (video, i) in to_delete {
        app.project
            .as_mut()
            .unwrap()
            .annotations
            .get_mut(&video)
            .unwrap()
            .remove(i);
    }
    None
}

fn export_labels_popup(_ui: &mut egui::Ui, app: &mut GlobalState, ctx: &egui::Context) {
    egui::Window::new("Export Labels")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Export Labels");
            if let Some(progress) = app.export_progress.as_ref() {
                let progress_value = progress.lock().unwrap();
                ui.add(
                    ProgressBar::new(*progress_value).text(if *progress_value == 1.0 {
                        "Done"
                    } else {
                        "Exporting labels..."
                    }),
                );
            }
            if ui.button("Close").clicked() {
                app.show_export_popup = false;
            }
        });
}

fn export_labels(
    video_path: &PathBuf,
    annotations: Vec<FrameAnnotation>,
) -> std::sync::Arc<std::sync::Mutex<f32>> {
    let progress = std::sync::Arc::new(std::sync::Mutex::new(0.0));
    let video_path = video_path.clone();
    let annotations = annotations.clone();
    let progress_clone = progress.clone();

    let export_dir = video_path.parent().unwrap().join(format!(
        "{}_exported_frames",
        video_path.file_stem().unwrap().to_string_lossy()
    ));

    let video_stem = video_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let csv_path = export_dir.join("labels.csv");

    let mut label_indices: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut next_index = 0;

    let mut classes = Vec::new();

    for annotation in &annotations {
        if !label_indices.contains_key(&annotation.label) {
            label_indices.insert(annotation.label.clone(), next_index);
            classes.push(annotation.label.clone());
            next_index += 1;
        }
    }

    let mut csv_content = format!("filename,{}\n", classes.join(","));

    if let Err(e) = fs::create_dir_all(&export_dir) {
        eprintln!("Failed to create export directory: {:?}", e);
        exit(1);
    }

    std::thread::spawn(move || {
        println!("Exporting labels to {:?}", video_path);
        let mut video = load_video(&video_path);
        let end_frame = annotations
            .iter()
            .map(|annotation| annotation.end_frame)
            .max()
            .unwrap_or(0);

        let mut frame_classes = Vec::with_capacity(classes.len());
        for i in 0..end_frame {
            *progress_clone.lock().unwrap() = i as f32 / end_frame as f32;

            for annotation in &annotations {
                if annotation.contains(i) {
                    frame_classes.push(label_indices[&annotation.label]);
                }
            }

            if frame_classes.is_empty() {
                continue;
            }

            let frame_filename = format!("{}_frame_{:05}.png", video_stem, i);
            let frame_path = export_dir.join(&frame_filename);

            let mut frame = opencv::core::Mat::default();
            video.set(videoio::CAP_PROP_POS_FRAMES, i as f64).unwrap();
            video.read(&mut frame).unwrap();

            imgcodecs::imwrite(
                frame_path.to_str().unwrap(),
                &frame,
                &opencv::core::Vector::<i32>::new(),
            )
            .unwrap();

            csv_content.push_str(&format!("{}", frame_filename));
            for class in &classes {
                if frame_classes.contains(&label_indices[class]) {
                    csv_content.push_str(&format!(",1"));
                } else {
                    csv_content.push_str(&format!(",0"));
                }
            }
            csv_content.push_str("\n");
            frame_classes.clear();
        }
        if let Err(e) = fs::write(&csv_path, csv_content) {
            eprintln!("Failed to write CSV file: {:?}", e);
            return;
        }
        *progress_clone.lock().unwrap() = 1.0;
    });

    progress
}

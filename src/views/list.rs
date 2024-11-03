use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
    thread,
};

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

pub struct ListView {
    progress: Option<Vec<Arc<Mutex<f32>>>>,
}

impl View for ListView {
    fn render(&mut self, ctx: &egui::Context, app: &mut GlobalState) -> Option<Box<dyn View>> {
        let mut next_view = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            next_view = list_ui(ui, app, self);
            label_list(ui, app);
            if app.show_export_popup {
                export_labels_popup(ui, app, ctx, self);
            }
        });
        next_view
    }
}

impl ListView {
    pub fn new() -> Self {
        Self { progress: None }
    }
}

fn list_ui(ui: &mut egui::Ui, app: &mut GlobalState, state: &mut ListView) -> Option<Box<dyn View>> {
    let mut next_view = None;
    ui.horizontal(|ui| {
        if ui.button("Home").clicked() {
            next_view = Some(Box::new(HomeView::new()) as Box<dyn View>);
        }
        if ui.button("Export").clicked() && state.progress.is_none() {
            let indicators = export_labels(app);
            state.progress = Some(indicators);
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

fn export_labels_popup(_ui: &mut egui::Ui, app: &mut GlobalState, ctx: &egui::Context, state: &ListView) {
    egui::Window::new("Export Labels")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Export Labels");
            if let Some(progress) = state.progress.as_ref() {
                for indicator in progress {
                    let progress_value = indicator.lock().unwrap();
                    ui.add(
                        ProgressBar::new(*progress_value).text(if *progress_value == 1.0 {
                            "Done"
                        } else {
                            "Exporting labels..."
                        }),
                    );
                }
            }
            if ui.button("Close").clicked() {
                app.show_export_popup = false;
            }
        });
}

fn export_labels(app: &GlobalState) -> Vec<Arc<Mutex<f32>>> {
    let mut progress = Vec::new();
    let project = app.project.as_ref().unwrap();
    for (video, annotations) in app
        .project
        .as_ref()
        .unwrap()
        .annotations
        .iter()
        .filter(|(_, annotations)| !annotations.is_empty())
    {
        let indicator = Arc::new(Mutex::new(0.0));
        progress.push(indicator.clone());
        let video_path = project.path.join(project.video_folder.join(video));
        let export_dir = project.path.join(project.labels_folder.join(video));
        let annotations = annotations.clone();
        thread::spawn(move || {
            export_labels_to_video(video_path, annotations, export_dir, indicator);
        });
    }
    progress
}

fn export_labels_to_video(
    video_path: PathBuf,
    annotations: Vec<FrameAnnotation>,
    export_dir: PathBuf,
    progress: Arc<Mutex<f32>>,
) {
    let video_stem = video_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let mut label_indices = HashMap::new();
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

    let csv_file = export_dir.join("labels.csv");

    println!("Exporting labels to {:?}", export_dir);
    let mut video = load_video(&video_path);
    let end_frame = annotations
        .iter()
        .map(|annotation| annotation.end_frame)
        .max()
        .unwrap_or(0);

    let mut frame_classes = Vec::with_capacity(classes.len());
    for i in 0..end_frame {
        *progress.lock().unwrap() = i as f32 / end_frame as f32;

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
    if let Err(e) = fs::write(&csv_file, csv_content) {
        eprintln!("Failed to write CSV file: {:?}", e);
    }
    *progress.lock().unwrap() = 1.0;
}

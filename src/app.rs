use std::path::PathBuf;

use crate::{project::Project, views};
use eframe::egui;
use opencv::{
    core::MatTraitConst,
    videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst},
};
use serde::{Deserialize, Serialize};

pub struct GlobalState {
    pub video_path: Option<PathBuf>,
    pub video_capture: Option<videoio::VideoCapture>,
    pub is_playing: bool,
    pub current_frame: Option<opencv::core::Mat>,
    pub annotations: Vec<FrameAnnotation>,
    pub show_export_popup: bool,
    pub export_progress: Option<std::sync::Arc<std::sync::Mutex<f32>>>,
    pub project: Option<Project>,
}

pub struct App {
    pub current_view: Box<dyn views::View>,
    pub global_state: GlobalState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameAnnotation {
    pub start_frame: u32,
    pub end_frame: u32,
    pub label: String,
}

impl FrameAnnotation {
    pub fn contains(&self, frame: u32) -> bool {
        frame >= self.start_frame && frame <= self.end_frame
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            current_view: Box::new(views::home::HomeView::new()),
            global_state: GlobalState {
                video_path: None,
                video_capture: None,
                is_playing: false,
                current_frame: None,
                annotations: Vec::new(),
                show_export_popup: false,
                export_progress: None,
                project: None,
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |_ui| {
            if let Some(next_view) = self.current_view.render(ctx, &mut self.global_state) {
                self.current_view = next_view;
            }
        });
    }
}

impl App {
    pub fn advance_frame(&mut self, step: u32) {
        if let Some(capture) = &mut self.global_state.video_capture {
            let mut frame = opencv::core::Mat::default();
            for _ in 0..step {
                if capture.read(&mut frame).unwrap() && !frame.empty() {
                    self.global_state.current_frame = Some(frame.clone());
                } else {
                    self.global_state.is_playing = false;
                    capture.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap();
                    break;
                }
            }
        }
    }

    pub fn previous_frame(&mut self, step: u32) {
        if let Some(capture) = &mut self.global_state.video_capture {
            if let Ok(pos) = capture.get(videoio::CAP_PROP_POS_FRAMES) {
                capture
                    .set(
                        videoio::CAP_PROP_POS_FRAMES,
                        (pos - step as f64 - 1.0).max(0.0),
                    )
                    .unwrap();
                self.advance_frame(1);
            }
        }
    }

    pub fn used_labels(&self) -> Vec<String> {
        self.global_state
            .annotations
            .iter()
            .map(|a| a.label.clone())
            .collect()
    }
}

impl GlobalState {
    pub fn used_labels(&self) -> Vec<String> {
        self.annotations.iter().map(|a| a.label.clone()).collect()
    }
}

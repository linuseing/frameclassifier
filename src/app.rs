use std::path::PathBuf;

use eframe::egui;
use opencv::{core::MatTraitConst, videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst}};
use crate::views;

pub struct App {
    pub current_view: View,
    pub video_path: Option<PathBuf>,
    pub video_capture: Option<videoio::VideoCapture>,
    pub is_playing: bool,
    pub current_frame: Option<opencv::core::Mat>,
    pub annotations: Vec<FrameRange>,
    pub current_start_frame: Option<u32>,
    pub current_end_frame: Option<u32>,
    pub show_label_popup: bool,
    pub label_input: String,
    pub show_export_popup: bool,
    pub export_progress: Option<std::sync::Arc<std::sync::Mutex<f32>>>,
}

#[derive(Debug, Clone)]
pub struct FrameRange {
    pub start_frame: u32,
    pub end_frame: u32,
    pub label: String,
}

impl FrameRange {
    pub fn contains(&self, frame: u32) -> bool {
        frame >= self.start_frame && frame <= self.end_frame
    }
}

pub enum View {
    Home,
    Label,
    List,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_view: View::Home,
            video_path: None,
            video_capture: None,
            is_playing: false,
            current_frame: None,
            annotations: Vec::new(),
            current_start_frame: None,
            current_end_frame: None,
            show_label_popup: false,
            label_input: String::new(),
            show_export_popup: false,
            export_progress: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.current_view {
            View::Home => views::home::render(ctx, self),
            View::Label => views::label::render(ctx, self),
            View::List => views::list::render(ctx, self),
        }
    }
}

impl App {
    pub fn advance_frame(&mut self, step: u32) {
        if let Some(capture) = &mut self.video_capture {
            let mut frame = opencv::core::Mat::default();
            for _ in 0..step {
                if capture.read(&mut frame).unwrap() && !frame.empty() {
                    self.current_frame = Some(frame.clone());
                } else {
                    self.is_playing = false;
                    capture.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap();
                    break;
                }
            }
        }
    }

    pub fn previous_frame(&mut self, step: u32) {
        if let Some(capture) = &mut self.video_capture {
            if let Ok(pos) = capture.get(videoio::CAP_PROP_POS_FRAMES) {
                capture.set(videoio::CAP_PROP_POS_FRAMES, (pos - step as f64 - 1.0).max(0.0)).unwrap();
                self.advance_frame(1);
            }
        }
    }

    pub fn used_labels(&self) -> Vec<String> {
        self.annotations.iter().map(|a| a.label.clone()).collect()
    }
}

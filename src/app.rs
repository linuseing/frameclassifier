use crate::{project::Project, views};
use eframe::egui;
use serde::{Deserialize, Serialize};

pub struct GlobalState {
    pub annotations: Vec<FrameAnnotation>,
    pub show_export_popup: bool,
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
                annotations: Vec::new(),
                show_export_popup: false,
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

impl GlobalState {
    pub fn used_labels(&self) -> Vec<String> {
        self.annotations.iter().map(|a| a.label.clone()).collect()
    }
}

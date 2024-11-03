use std::path::PathBuf;

use crate::{
    app::{FrameAnnotation, GlobalState},
    utils::load_video,
};
use egui_autocomplete::AutoCompleteTextEdit;
use opencv::{
    core::{self, MatTraitConst, MatTraitConstManual},
    imgproc,
    videoio::{self, VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst},
};

use super::{home::HomeView, list::ListView, View};

pub struct LabelView {
    capture: VideoCapture,
    current_frame: Option<core::Mat>,
    is_playing: bool,
    current_start_frame: Option<u32>,
    current_end_frame: Option<u32>,
    show_label_popup: bool,
    label_input: String,
    video_name: String,
}

impl View for LabelView {
    fn render(&mut self, ctx: &egui::Context, app: &mut GlobalState) -> Option<Box<dyn View>> {
        let mut next_view = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Back").clicked() {
                    next_view = Some(Box::new(HomeView::new()) as Box<dyn View>);
                }
                ui.heading("Label");
            });
            let next = playback_ui(ui, self);
            if next.is_some() {
                next_view = next;
            }
            video_ui(ui, app, self);
            label_ui(ui, app, self);
            label_popup(ui, ctx, self, app);
            let next = controls(ctx, self);
            if next.is_some() {
                next_view = next;
            }
        });
        next_view
    }
}

impl LabelView {
    pub fn from_video_path(path: PathBuf) -> Self {
        let capture = load_video(&path);
        Self {
            capture,
            current_frame: None,
            is_playing: false,
            current_start_frame: None,
            current_end_frame: None,
            show_label_popup: false,
            label_input: String::new(),
            video_name: path.file_name().unwrap().to_string_lossy().to_string(),
        }
    }
}

fn controls(ctx: &egui::Context, state: &mut LabelView) -> Option<Box<dyn View>> {
    let mut next_view = None;
    if !state.show_label_popup {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                state.is_playing = !state.is_playing;
            } else if i.key_pressed(egui::Key::ArrowRight) {
                let step = if i.modifiers.shift { 10 } else { 1 };
                state.advance_frame(step);
            } else if i.key_pressed(egui::Key::ArrowLeft) {
                let step = if i.modifiers.shift { 10 } else { 1 };
                state.previous_frame(step);
            } else if i.key_pressed(egui::Key::Escape) {
                next_view = Some(Box::new(HomeView::new()) as Box<dyn View>);
            } else if i.key_pressed(egui::Key::S) {
                let current_frame = state.capture.get(videoio::CAP_PROP_POS_FRAMES).unwrap() as u32;
                if let Some(end_frame) = state.current_end_frame {
                    if current_frame <= end_frame {
                        state.current_start_frame = Some(current_frame);
                    }
                } else {
                    state.current_start_frame = Some(current_frame);
                }
            } else if i.key_pressed(egui::Key::E) {
                let current_frame = state.capture.get(videoio::CAP_PROP_POS_FRAMES).unwrap() as u32;
                if let Some(start_frame) = state.current_start_frame {
                    if current_frame >= start_frame {
                        state.current_end_frame = Some(current_frame);
                    }
                } else {
                    state.current_end_frame = Some(current_frame);
                }
            } else if i.key_pressed(egui::Key::L) {
                if state.current_start_frame.is_some() && state.current_end_frame.is_some() {
                    state.show_label_popup = true;
                }
            }
        });
    }
    next_view
}

pub fn playback_ui(ui: &mut egui::Ui, state: &mut LabelView) -> Option<Box<dyn View>> {
    let mut next_view = None;
    ui.horizontal(|ui| {
        if ui.button("Previous").clicked() {
            state.previous_frame(1);
        }
        if state.is_playing {
            if ui.button("Pause").clicked() {
                state.is_playing = false;
            }
        } else {
            if ui.button("Play").clicked() {
                state.is_playing = true;
            }
        }
        if ui.button("Next").clicked() {
            state.advance_frame(1);
        }
        seeker_ui(ui, state);
        ui.separator();
        if ui.button("View all labels").clicked() {
            next_view = Some(Box::new(ListView::new()) as Box<dyn View>);
        }
    });
    next_view
}

pub fn seeker_ui(ui: &mut egui::Ui, state: &mut LabelView) {
    let total_frames = state.capture.get(videoio::CAP_PROP_FRAME_COUNT).unwrap();
    let current_frame = state.capture.get(videoio::CAP_PROP_POS_FRAMES).unwrap();

    let mut frame_position = current_frame;
    if ui
        .add(egui::Slider::new(&mut frame_position, 0.0..=total_frames).show_value(true))
        .changed()
    {
        state
            .capture
            .set(videoio::CAP_PROP_POS_FRAMES, frame_position)
            .unwrap();
        state.advance_frame(1);
    }
}

pub fn video_ui(ui: &mut egui::Ui, _app: &mut GlobalState, state: &mut LabelView) {
    if state.current_frame.is_none() {
        state.advance_frame(1);
    }

    let current_frame = state.current_frame.as_ref().unwrap();
    let mut rgb_frame = core::Mat::default();
    imgproc::cvt_color(current_frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB, 0).unwrap();

    let frame_size = rgb_frame.size().unwrap();
    let available_size = ui.available_size();
    let scale = (available_size.x / frame_size.width as f32)
        .min(available_size.y / frame_size.height as f32);
    let target_width = (frame_size.width as f32 * scale) as i32;
    let target_height = (frame_size.height as f32 * scale) as i32;

    let mut resized_frame = core::Mat::default();
    imgproc::resize(
        &rgb_frame,
        &mut resized_frame,
        core::Size::new(target_width, target_height),
        0.0,
        0.0,
        imgproc::INTER_LINEAR,
    )
    .unwrap();

    let resized_frame_size = resized_frame.size().unwrap();
    let image_buffer = resized_frame.data_bytes().unwrap();

    let texture = ui.ctx().load_texture(
        "current_frame",
        egui::ColorImage::from_rgb(
            [
                resized_frame_size.width as usize,
                resized_frame_size.height as usize,
            ],
            image_buffer,
        ),
        egui::TextureOptions::default(),
    );

    ui.image(&texture);
}

fn label_ui(ui: &mut egui::Ui, app: &mut GlobalState, state: &mut LabelView) {
    ui.horizontal(|ui| {
        let current_frame = state.capture.get(videoio::CAP_PROP_POS_FRAMES).unwrap();

        ui.label(&format!(
            "Labels: {}",
            app.project
                .as_mut()
                .unwrap()
                .annotations
                .entry(state.video_name.clone())
                .or_insert(vec![])
                .iter()
                .filter(|annotation| annotation.contains(current_frame as u32))
                .map(|annotation| annotation.label.clone())
                .collect::<Vec<String>>()
                .join(", ")
        ));
    });
    ui.label(&format!(
        "start: {}",
        state
            .current_start_frame
            .map_or("-".to_string(), |start| start.to_string())
    ));
    ui.label(&format!(
        "end: {}",
        state
            .current_end_frame
            .map_or("-".to_string(), |end| end.to_string())
    ));
}

fn label_popup(
    _ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut LabelView,
    app: &mut GlobalState,
) {
    if state.show_label_popup {
        egui::Window::new("Enter Label")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let mut suggestions = app
                    .project
                    .as_ref()
                    .unwrap()
                    .used_labels
                    .iter()
                    .map(|a| a.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .filter(|label| label.starts_with(&state.label_input))
                    .collect::<Vec<_>>();

                suggestions.sort();

                let widget = ui.add(
                    AutoCompleteTextEdit::new(&mut state.label_input, suggestions)
                        .max_suggestions(10)
                        .highlight_matches(true),
                );

                if !widget.has_focus() {
                    widget.request_focus();
                }

                ui.horizontal(|ui| {
                    if ui.button("Set").clicked() {
                        close_label_popup(state, app);
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_label_popup = false;
                        state.label_input.clear();
                    }
                });
            });
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                state.show_label_popup = false;
                state.label_input.clear();
            } else if i.key_pressed(egui::Key::Enter) {
                close_label_popup(state, app);
            }
        });
    }
}

fn close_label_popup(state: &mut LabelView, app: &mut GlobalState) {
    if state.current_end_frame.is_none() {
        return;
    }
    if state.current_start_frame.is_none() {
        return;
    }
    if state.label_input.is_empty() {
        return;
    }

    let start_frame = state.current_start_frame.take().unwrap();
    let end_frame = state.current_end_frame.take().unwrap();
    let annotation = FrameAnnotation {
        start_frame,
        end_frame,
        label: state.label_input.clone(),
    };

    app.project
        .as_mut()
        .unwrap()
        .annotations
        .entry(state.video_name.clone())
        .or_insert(vec![])
        .push(annotation);

    app.project
        .as_mut()
        .unwrap()
        .used_labels
        .insert(state.label_input.clone());
    state.label_input.clear();
    state.show_label_popup = false;
}

impl LabelView {
    pub fn advance_frame(&mut self, step: u32) {
        let mut frame = opencv::core::Mat::default();
        for _ in 0..step {
            if self.capture.read(&mut frame).unwrap() && !frame.empty() {
                self.current_frame = Some(frame.clone());
            } else {
                self.is_playing = false;
                self.capture.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap();
                break;
            }
        }
    }

    pub fn previous_frame(&mut self, step: u32) {
        if let Ok(pos) = self.capture.get(videoio::CAP_PROP_POS_FRAMES) {
            self.capture
                .set(
                    videoio::CAP_PROP_POS_FRAMES,
                    (pos - step as f64 - 1.0).max(0.0),
                )
                .unwrap();
            self.advance_frame(1);
        }
    }
}

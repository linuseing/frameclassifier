use crate::app::{App, FrameRange, View};
use egui_autocomplete::AutoCompleteTextEdit;
use opencv::{core::{self, MatTraitConst, MatTraitConstManual}, imgproc, videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst}};

pub fn render(ctx: &egui::Context, _app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Label");
        playback_ui(ui, _app);
        video_ui(ui, _app);
        label_ui(ui, _app);
        label_popup(ui, _app, ctx);
        controls(ctx, _app);
    });
}

fn controls(ctx: &egui::Context, app: &mut App) {
    if !app.show_label_popup {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                app.is_playing = !app.is_playing;
            } else if i.key_pressed(egui::Key::ArrowRight) {
                let step = if i.modifiers.shift { 10 } else { 1 };
                app.advance_frame(step);
            } else if i.key_pressed(egui::Key::ArrowLeft) {
                let step = if i.modifiers.shift { 10 } else { 1 };
                app.previous_frame(step);
            } else if i.key_pressed(egui::Key::Escape) {
                app.is_playing = false;
                app.video_capture = None;
                app.video_path = None;
            } else if i.key_pressed(egui::Key::S) {
                let current_frame = app.video_capture.as_ref().unwrap().get(videoio::CAP_PROP_POS_FRAMES).unwrap() as u32;
                if let Some(end_frame) = app.current_end_frame {
                    if current_frame <= end_frame {
                        app.current_start_frame = Some(current_frame);
                    }
                } else {
                    app.current_start_frame = Some(current_frame);
                }
            } else if i.key_pressed(egui::Key::E) {
                let current_frame = app.video_capture.as_ref().unwrap().get(videoio::CAP_PROP_POS_FRAMES).unwrap() as u32;
                if let Some(start_frame) = app.current_start_frame {
                    if current_frame >= start_frame {
                        app.current_end_frame = Some(current_frame);
                    }
                } else {
                    app.current_end_frame = Some(current_frame);
                }
            } else if i.key_pressed(egui::Key::L) {
                if app.current_start_frame.is_some() && app.current_end_frame.is_some() {
                    app.show_label_popup = true;
                }
            }
        });
    }
}

pub fn playback_ui(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        if ui.button("Previous").clicked() {
            app.previous_frame(1);
        }
        if app.is_playing {
            if ui.button("Pause").clicked() {
                app.is_playing = false;
            }
        } else {
            if ui.button("Play").clicked() {
                app.is_playing = true;
            }
        }
        if ui.button("Next").clicked() {
            app.advance_frame(1);
        }
        seeker_ui(ui, app);
        ui.separator();
        if ui.button("View all labels").clicked() {
            app.current_view = View::List;
        }
    });
}

pub fn seeker_ui(ui: &mut egui::Ui, app: &mut App) {
    if let Some(capture) = &mut app.video_capture {
        let total_frames = capture.get(videoio::CAP_PROP_FRAME_COUNT).unwrap();
        let current_frame = capture.get(videoio::CAP_PROP_POS_FRAMES).unwrap();
        
        let mut frame_position = current_frame;
        if ui.add(
            egui::Slider::new(&mut frame_position, 0.0..=total_frames)
                .show_value(true)
        ).changed() {
            capture.set(videoio::CAP_PROP_POS_FRAMES, frame_position).unwrap();
            app.advance_frame(1);
        }
    }
}

pub fn video_ui(ui: &mut egui::Ui, app: &mut App) {
    if app.current_frame.is_none() {
        if let Some(_capture) = &mut app.video_capture {
            app.advance_frame(1);
        }
    }

    let current_frame = app.current_frame.as_ref().unwrap();
    let mut rgb_frame = core::Mat::default();
    imgproc::cvt_color(current_frame, &mut rgb_frame, imgproc::COLOR_BGR2RGB, 0).unwrap();
    
    let frame_size = rgb_frame.size().unwrap();
    let available_size = ui.available_size();
    let scale = (available_size.x / frame_size.width as f32)
        .min(available_size.y / frame_size.height as f32);
    let target_width = (frame_size.width as f32 * scale) as i32;
    let target_height = (frame_size.height as f32 * scale) as i32;
    
    let mut resized_frame = core::Mat::default();
    imgproc::resize(&rgb_frame, &mut resized_frame, core::Size::new(target_width, target_height), 0.0, 0.0, imgproc::INTER_LINEAR).unwrap();
    
    let resized_frame_size = resized_frame.size().unwrap();
    let image_buffer = resized_frame.data_bytes().unwrap();
    
    let texture = ui.ctx().load_texture(
        "current_frame",
        egui::ColorImage::from_rgb([resized_frame_size.width as usize, resized_frame_size.height as usize], image_buffer),
        egui::TextureOptions::default(),
    );
    
    ui.image(&texture);
}

fn label_ui(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        let current_frame = app.video_capture.as_ref().unwrap().get(videoio::CAP_PROP_POS_FRAMES).unwrap();
        ui.label(
            &format!(
                "Labels: {}",
                app.annotations.iter().filter(|annotation| annotation.contains(current_frame as u32)).map(|annotation| annotation.label.clone()).collect::<Vec<String>>().join(", ")
            ),
        );
    });
    ui.label(&format!("start: {}", app.current_start_frame.map_or("-".to_string(), |start| start.to_string())));
    ui.label(&format!("end: {}", app.current_end_frame.map_or("-".to_string(), |end| end.to_string())));
}

fn label_popup(_ui: &mut egui::Ui, app: &mut App, ctx: &egui::Context) {
    if app.show_label_popup {
        egui::Window::new("Enter Label")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let mut suggestions = app.annotations.iter()
                    .map(|a| a.label.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .filter(|label| label.starts_with(&app.label_input))
                    .collect::<Vec<_>>();
                suggestions.sort();
                let widget = ui.add(
                    AutoCompleteTextEdit::new(&mut app.label_input, suggestions)
                    .max_suggestions(10)
                    .highlight_matches(true)
                );

                if !widget.has_focus() {
                    widget.request_focus();
                }

                ui.horizontal(|ui| {
                    if ui.button("Set").clicked() {
                        close_label_popup(app);
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_label_popup = false;
                        app.label_input.clear();
                    }
                });
            });
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    app.show_label_popup = false;
                    app.label_input.clear();
                } else if i.key_pressed(egui::Key::Enter) {
                    close_label_popup(app);
                }
            });
    }
}

fn close_label_popup(app: &mut App) {
    if app.current_end_frame.is_none() {
        return;
    }
    if app.current_start_frame.is_none() {
        return;
    }
    if app.label_input.is_empty() {
        return;
    }

    let start_frame = app.current_start_frame.take().unwrap();
    let end_frame = app.current_end_frame.take().unwrap();
    let annotation = FrameRange {
        start_frame,
        end_frame,
        label: app.label_input.clone(),
    };
    app.annotations.push(annotation);
    app.label_input.clear();
    app.show_label_popup = false;
}

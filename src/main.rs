use eframe::{egui, App, CreationContext};
use opencv::{core, imgcodecs, imgproc, prelude::*, videoio};
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

struct VideoApp {
    video_path: Option<PathBuf>,
    current_frame: Option<core::Mat>,
    capture: Option<videoio::VideoCapture>,
    is_playing: bool,
    last_frame_time: Instant,
    frame_duration: Duration,
    start_frame: Option<f64>,
    end_frame: Option<f64>,
    show_label_popup: bool,
    label_input: String,
}

impl Default for VideoApp {
    fn default() -> Self {
        Self {
            video_path: None,
            current_frame: None,
            capture: None,
            is_playing: false,
            last_frame_time: Instant::now(),
            frame_duration: Duration::from_secs_f32(1.0 / 30.0), // Default to 30 FPS
            start_frame: None,
            end_frame: None,
            show_label_popup: false,
            label_input: String::new(),
        }
    }
}

impl App for VideoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.show_label_popup {
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Space) {
                    self.is_playing = !self.is_playing;
                } else if i.key_pressed(egui::Key::ArrowRight) {
                    self.advance_frame();
                    self.last_frame_time = Instant::now();
                } else if i.key_pressed(egui::Key::ArrowLeft) {
                    self.previous_frame();
                    self.last_frame_time = Instant::now();
                } else if i.key_pressed(egui::Key::Escape) {
                    self.is_playing = false;
                    self.capture = None;
                    self.video_path = None;
                } else if i.key_pressed(egui::Key::Q) {
                    self.is_playing = false;
                    std::process::exit(0);
                } else if i.key_pressed(egui::Key::S) {
                    if let Some(capture) = &self.capture {
                        if let Ok(current_frame) = capture.get(videoio::CAP_PROP_POS_FRAMES) {
                            self.start_frame = Some(current_frame);
                        }
                    }
                } else if i.key_pressed(egui::Key::E) {
                    if let Some(capture) = &self.capture {
                        if let Ok(current_frame) = capture.get(videoio::CAP_PROP_POS_FRAMES) {
                            self.end_frame = Some(current_frame);
                        }
                    }
                } else if i.key_pressed(egui::Key::O) {
                    if let Some(path) = FileDialog::new().add_filter("Video", &["mp4", "avi", "mov"]).pick_file() {
                        self.load_video(path);
                    }
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Video Playback");

            ui.horizontal(|ui| {
                if ui.button("Open Video").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("Video", &["mp4", "avi", "mov"]).pick_file() {
                        self.load_video(path);
                    }
                }

                let play_pause_label = if self.is_playing { "Pause" } else { "Play" };
                if ui.button(play_pause_label).clicked() {
                    self.is_playing = !self.is_playing;
                }

                if ui.button("Set Start Frame").clicked() {
                    if let Some(capture) = &self.capture {
                        if let Ok(current_frame) = capture.get(videoio::CAP_PROP_POS_FRAMES) {
                            self.start_frame = Some(current_frame);
                        }
                    }
                }

                if ui.button("Set End Frame").clicked() {
                    if let Some(capture) = &self.capture {
                        if let Ok(current_frame) = capture.get(videoio::CAP_PROP_POS_FRAMES) {
                            self.end_frame = Some(current_frame);
                        }
                    }
                }

                // Button to export frames
                if ui.button("Export Frames").clicked() {
                    self.show_label_popup = true;
                }

                if let Some(capture) = &mut self.capture {
                    let total_frames = capture.get(videoio::CAP_PROP_FRAME_COUNT).unwrap();
                    let current_frame = capture.get(videoio::CAP_PROP_POS_FRAMES).unwrap();
                    
                    let mut frame_position = current_frame;
                    if ui.add(
                        egui::Slider::new(&mut frame_position, 0.0..=total_frames)
                            .show_value(true)
                    ).changed() {
                        capture.set(videoio::CAP_PROP_POS_FRAMES, frame_position).unwrap();
                        self.advance_frame();
                        self.last_frame_time = Instant::now();
                    }
                }

                if let Some(start) = self.start_frame {
                    ui.label(format!("Start Frame: {}", start as u64));
                }
                if let Some(end) = self.end_frame {
                    ui.label(format!("End Frame: {}", end as u64));
                }
            });

            if self.is_playing {
                let now = Instant::now();
                if now.duration_since(self.last_frame_time) >= self.frame_duration {
                    self.advance_frame();
                    self.last_frame_time = now;
                }
            }

            if let Some(current_frame) = &self.current_frame {
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
        });

        if self.is_playing {
            ctx.request_repaint();
        }

        if self.show_label_popup {
            egui::Window::new("Enter Label")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.text_edit_singleline(&mut self.label_input);
                    ui.horizontal(|ui| {
                        if ui.button("Export").clicked() {
                            self.export_frames_with_label();
                            self.show_label_popup = false;
                            self.label_input.clear();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_label_popup = false;
                            self.label_input.clear();
                        }
                    });
                });
        }
    }
}

impl VideoApp {
    fn load_video(&mut self, path: PathBuf) {
        if let Ok(capture) = videoio::VideoCapture::from_file(&path.to_string_lossy(), videoio::CAP_ANY) {
            if let Ok(true) = capture.is_opened() {
                self.video_path = Some(path);
                self.capture = Some(capture);
                self.is_playing = true;

                if let Some(fps) = self.capture.as_ref().and_then(|c| c.get(videoio::CAP_PROP_FPS).ok()) {
                    self.frame_duration = Duration::from_secs_f64(1.0 / fps);
                }
                self.advance_frame();
            } else {
                eprintln!("Failed to open video file");
            }
        } else {
            eprintln!("Could not load video from path: {:?}", path);
        }
    }

    fn advance_frame(&mut self) {
        if let Some(capture) = &mut self.capture {
            let mut frame = core::Mat::default();
            if capture.read(&mut frame).unwrap() && !frame.empty() {
                self.current_frame = Some(frame);
            } else {
                self.is_playing = false;
                capture.set(videoio::CAP_PROP_POS_FRAMES, 0.0).unwrap();
            }
        }
    }

    fn previous_frame(&mut self) {
        if let Some(capture) = &mut self.capture {
            if let Ok(pos) = capture.get(videoio::CAP_PROP_POS_FRAMES) {
                capture.set(videoio::CAP_PROP_POS_FRAMES, (pos - 2.0).max(0.0)).unwrap();
                self.advance_frame();
            }
        }
    }

    fn export_frames_with_label(&mut self) {
        if let (Some(start_frame), Some(end_frame), Some(capture), Some(video_path)) =
            (self.start_frame, self.end_frame, &mut self.capture, &self.video_path)
        {
            let export_dir = video_path
                .parent()
                .unwrap()
                .join(format!("{}-{}_exported_frames", video_path.file_stem().unwrap().to_string_lossy(), self.label_input));

            let video_stem = video_path.file_stem().unwrap().to_string_lossy().to_string();

            if let Err(e) = fs::create_dir_all(&export_dir) {
                eprintln!("Failed to create export directory: {:?}", e);
                return;
            }

            let csv_path = export_dir.join("labels.csv");
            let mut csv_content = format!("filename,{}\n", self.label_input);

            for frame_number in start_frame as i32..=end_frame as i32 {
                capture.set(videoio::CAP_PROP_POS_FRAMES, frame_number as f64).unwrap();
                let mut frame = core::Mat::default();
                if capture.read(&mut frame).unwrap() && !frame.empty() {
                    let frame_filename = format!("{}-{}_frame_{:05}.png", video_stem, self.label_input, frame_number);
                    let frame_path = export_dir.join(&frame_filename);
                    imgcodecs::imwrite(frame_path.to_str().unwrap(), &frame, &opencv::core::Vector::<i32>::new()).unwrap();
                    
                    csv_content.push_str(&format!("{},1\n", frame_filename));
                }
            }
            if let Err(e) = fs::write(&csv_path, csv_content) {
                eprintln!("Failed to write CSV file: {:?}", e);
                return;
            }

            eprintln!("Frames and labels exported to: {:?}", export_dir);
        }
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let _ =eframe::run_native(
        "Video Playback",
        options,
        Box::new(|_cc: &CreationContext| Box::new(VideoApp::default())),
    );
}


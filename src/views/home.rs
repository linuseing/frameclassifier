use std::{collections::HashSet, fs};

use rfd::FileDialog;

use crate::{
    app::GlobalState,
    project::{load_project_from_path, Project},
};

use super::{label::LabelView, list::ListView, View};

pub struct HomeView {
    show_project_name_dialog: bool,
    new_project_name: String,
    show_labels_popup: bool,
}

impl View for HomeView {
    fn render(&mut self, ctx: &egui::Context, app: &mut GlobalState) -> Option<Box<dyn View>> {
        let mut next_view = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Home");

            ui.horizontal(|ui| {
                if ui.button("Load Project").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        app.project = Some(load_project_from_path(&path));
                    }
                }

                if ui.button("New Project").clicked() {
                    self.show_project_name_dialog = true;
                }
            });

            if let Some(project) = app.project.as_ref() {
                ui.horizontal(|ui| {
                    if ui.button("show labels").clicked() {
                        self.show_labels_popup = true;
                    }
                    if ui.button("Save").clicked() {
                        let _ = fs::write(
                            project.path.join("project.json"),
                            serde_json::to_string(&project.config()).unwrap(),
                        );
                    }
                    if ui.button("Export").clicked() {
                        next_view = Some(Box::new(ListView::new()) as Box<dyn View>);
                    }
                });

                ui.label(format!("{}", project.path.display()));

                let video_files = std::fs::read_dir(&project.path.join(&project.video_folder))
                    .unwrap()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.path().extension().map_or(false, |ext| {
                            ext.eq_ignore_ascii_case("mp4")
                                || ext.eq_ignore_ascii_case("avi")
                                || ext.eq_ignore_ascii_case("mov")
                        })
                    })
                    .collect::<Vec<_>>();

                for video in video_files {
                    ui.horizontal(|ui| {
                        ui.label(video.file_name().to_string_lossy().to_string());

                        if ui.button("Label").clicked() {
                            next_view =
                                Some(Box::new(LabelView::from_video_path(video.path()))
                                    as Box<dyn View>);
                        }
                    });
                }
            }

            if self.show_project_name_dialog {
                new_project_popup(ui, self, ctx, app);
            }
            if self.show_labels_popup {
                labels_popup(ui, self, ctx, app);
            }
        });
        next_view
    }
}

impl HomeView {
    pub fn new() -> Self {
        Self {
            show_project_name_dialog: false,
            new_project_name: "".to_string(),
            show_labels_popup: false,
        }
    }
}

fn new_project_popup(
    _ui: &mut egui::Ui,
    app: &mut HomeView,
    ctx: &egui::Context,
    app_state: &mut GlobalState,
) {
    egui::Window::new("New Project")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("New Project");
            ui.text_edit_singleline(&mut app.new_project_name);
            if ui.button("Close").clicked() {
                app.show_project_name_dialog = false;
            }
            if ui.button("Create").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    let project_path = path.join(app.new_project_name.clone());
                    let project = Project::with_root(project_path.clone());

                    std::fs::create_dir_all(&project.path).unwrap();
                    std::fs::create_dir_all(&project.path.join("video")).unwrap();
                    std::fs::create_dir_all(&project.path.join("labels")).unwrap();

                    let config = project.config();
                    let _ = fs::write(
                        project_path.join("project.json"),
                        serde_json::to_string(&config).unwrap(),
                    );

                    app_state.project = Some(project);
                    app.show_project_name_dialog = false;
                }
            }
        });
}

fn labels_popup(
    _ui: &mut egui::Ui,
    app: &mut HomeView,
    ctx: &egui::Context,
    app_state: &mut GlobalState,
) {
    egui::Window::new("Labels")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            for label in app_state.project.as_ref().unwrap().used_labels.iter() {
                ui.label(label);
            }
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    app.show_labels_popup = false;
                }
                if ui.button("Reload").clicked() {
                    let mut used_labels = HashSet::new();
                    for (_, annotations) in app_state.project.as_mut().unwrap().annotations.iter() {
                        for annotation in annotations.iter() {
                            used_labels.insert(annotation.label.clone());
                        }
                    }
                    app_state.project.as_mut().unwrap().used_labels = used_labels;
                }
            });
        });
}

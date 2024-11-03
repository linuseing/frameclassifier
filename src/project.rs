use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

type Label = String;

#[derive(Debug, Clone)]
pub struct Project {
    pub path: PathBuf,
    pub video_folder: PathBuf,
    pub labels_folder: PathBuf,
    pub annotations: HashMap<Label, Vec<FrameAnnotation>>,
    pub used_labels: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub video_folder: String,
    pub labels_folder: String,
    pub used_labels: Vec<String>,
    pub annotations: HashMap<String, Vec<FrameAnnotation>>,
}

impl From<ProjectConfig> for Project {
    fn from(config: ProjectConfig) -> Self {
        Project {
            path: PathBuf::new(),
            video_folder: PathBuf::from(config.video_folder),
            labels_folder: PathBuf::from(config.labels_folder),
            annotations: config.annotations,
            used_labels: config.used_labels,
        }
    }
}

fn load_project_from_path(path: &PathBuf) -> Project {
    let config_path = path.join("classifier_project.json");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: ProjectConfig = serde_json::from_str(&config).unwrap();
    Project::from(config)
}
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::app::FrameAnnotation;

pub type Label = String;
pub type Video = String;

#[derive(Debug, Clone)]
pub struct Project {
    pub path: PathBuf,
    pub video_folder: PathBuf,
    pub labels_folder: PathBuf,
    pub annotations: HashMap<Label, Vec<FrameAnnotation>>,
    pub used_labels: HashSet<Label>,
}

impl Project {
    pub fn with_root(root: PathBuf) -> Self {
        let video_folder = PathBuf::from("video");
        let labels_folder = PathBuf::from("labels");

        Project {
            path: root,
            video_folder,
            labels_folder,
            annotations: HashMap::new(),
            used_labels: HashSet::new(),
        }
    }

    pub fn config(&self) -> ProjectConfig {
        ProjectConfig {
            video_folder: self.video_folder.to_str().unwrap().to_string(),
            labels_folder: self.labels_folder.to_str().unwrap().to_string(),
            used_labels: self.used_labels.clone().into_iter().collect(),
            annotations: self.annotations.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub video_folder: String,
    pub labels_folder: String,
    pub used_labels: Vec<Label>,
    pub annotations: HashMap<Video, Vec<FrameAnnotation>>,
}

impl Project {
    fn from(config: ProjectConfig, root: PathBuf) -> Self {
        Project {
            path: root,
            video_folder: PathBuf::from(config.video_folder),
            labels_folder: PathBuf::from(config.labels_folder),
            annotations: config.annotations,
            used_labels: config.used_labels.into_iter().collect(),
        }
    }
}

pub fn load_project_from_path(path: &PathBuf) -> Project {
    let config_path = path.join("project.json");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: ProjectConfig = serde_json::from_str(&config).unwrap();
    Project::from(config, path.to_path_buf())
}

use std::path::PathBuf;

use opencv::videoio::{self, VideoCapture};

pub fn load_video(path: &PathBuf) -> VideoCapture {
    videoio::VideoCapture::from_file(path.to_string_lossy().as_ref(), videoio::CAP_ANY).unwrap()
}

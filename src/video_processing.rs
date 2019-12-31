use std::fs;
use std::process::{Command, Stdio};
use std::fmt::{self, Display};

pub struct Video {
    pub source: VideoSource,
    pub length_seconds: Option<f32>,
    pub get_loud_or_silent_frames_by_second: Option<Vec<bool>>,
}
impl Video {
    pub fn new(source: VideoSource) -> Video {
        Video {
            source,
            length_seconds: None,
            get_loud_or_silent_frames_by_second: None,
        }
    }

    /// Analyze video length and sound loud/silent frames
    pub fn analyze_sound(&mut self) -> Result<(), VideoProcessingError> {
        
        unimplemented!();
    }

    /// Call ffprobe to determina video duration.
    /// It looks like this:
    /// 
    /// ffprobe -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 $FILE
    /// 
    /// Output:
    /// 2838.919000
    /// 
    /// Source:
    /// https://superuser.com/questions/650291/how-to-get-video-duration-in-seconds
    fn get_video_length_in_seconds(&self) -> Result<(), VideoProcessingError> {
        // let duration_command = Command::new("ffprobe")
        //         .arg("-show_entries")
        //         .arg("format=duration")
        //         .arg("-of default=noprint_wrappers=1:nokey=1")
        //         .arg(self.filename)
        //         .stdout(Stdio::piped())
        //         .stdin(Stdio::null())
        //         .stderr(Stdio::null())
        //         .output()
        //         .expect("Failed to spawn sound extract process.");
        //     let output = sound.stdout;
        unimplemented!();
    }
}

pub enum VideoProcessingError {

}
impl Display for VideoProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown error.")
    }
}

/// Source of video. Either path to file or stream
/// of bytes from stdin.
pub enum VideoSource {
    FilePath(String),
    StdinStream(Vec<u8>),
}

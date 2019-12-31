use std::fmt::{self, Display};
use std::io::{BufWriter, Write, prelude::*};
use std::process::{Command, Stdio};

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
        self.get_video_length_in_seconds()?;

        Ok(())
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
    fn get_video_length_in_seconds(&mut self) -> Result<(), VideoProcessingError> {
        let parameter_source = match &self.source {
            VideoSource::FilePath(s) => s,
            VideoSource::StdinStream(_) => "-",
        };
        let parameter_stdin = match &self.source {
            VideoSource::FilePath(_) => Stdio::null(),
            VideoSource::StdinStream(_) => Stdio::piped(),
        };
        let mut duration_command = Command::new("ffprobe")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg(parameter_source)
            .stdout(Stdio::piped())
            .stdin(parameter_stdin)
            .stderr(Stdio::null())
            .spawn()?;
        match &self.source {
            VideoSource::StdinStream(stream) => {
                // Send stdin contents to the process
                let mut command_stdin = duration_command.stdin.unwrap();
                let mut writer = BufWriter::new(&mut command_stdin);
                writer.write_all(stream)?;
            },
            _ => {}
        }
        // Wait for duration command to end
        let mut output = duration_command.stdout.unwrap();
        let mut output_buffer: Vec<u8> = Vec::new();
        output.read_to_end(&mut output_buffer)?;
        let output_as_string = String::from_utf8(output_buffer)?;
        self.length_seconds = Some(output_as_string.trim().parse()?);
        Ok(())
    }
}

pub enum VideoProcessingError {
    ExternalIOError(std::io::Error),
    FaieldToReadOutput(std::string::FromUtf8Error),
    BadOutputFromFFprobe(std::num::ParseFloatError),
}
impl Display for VideoProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason: String = match self {
            VideoProcessingError::ExternalIOError(e) => {
                format!("Failed to spawn process of ffmpeg/ffprobe: {}", e)
            },
            VideoProcessingError::FaieldToReadOutput(e) => {
                format!("Failed to read output of ffmpeg/ffprobe process, output was not valid utf-8: {}", e)
            },
            VideoProcessingError::BadOutputFromFFprobe(e) => {
                format!("Bad output from ffprobe, failed to parse to f32: {}", e)
            }
        };
        write!(f, "An error occured while processing video: {}", reason)
    }
}
impl From<std::io::Error> for VideoProcessingError {
    fn from(err: std::io::Error) -> Self {
        VideoProcessingError::ExternalIOError(err)
    }
}
impl From<std::string::FromUtf8Error> for VideoProcessingError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        VideoProcessingError::FaieldToReadOutput(err)
    }
}
impl From<std::num::ParseFloatError> for VideoProcessingError {
    fn from(err: std::num::ParseFloatError) -> Self {
        VideoProcessingError::BadOutputFromFFprobe(err)
    }
}

/// Source of video. Either path to file or stream
/// of bytes from stdin.
pub enum VideoSource {
    FilePath(String),
    StdinStream(Vec<u8>),
}

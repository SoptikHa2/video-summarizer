use crate::cli::Cli;
use minimp3::{Decoder, Error};
use std::fmt::{self, Display};
use std::io::{prelude::*, BufWriter, Write};
use std::process::{Command, Stdio};

pub struct Video {
    pub source: VideoSource,
    pub length_seconds: Option<f32>,
    pub silent_frames: Option<Vec<bool>>,
}
impl Video {
    pub fn new(source: VideoSource) -> Video {
        Video {
            source,
            length_seconds: None,
            silent_frames: None,
        }
    }

    /// Analyze video length and sound loud/silent frames
    pub fn analyze_sound(&mut self, settings: &Cli) -> Result<(), VideoProcessingError> {
        if !settings.quiet {
            eprintln!("Analyzing video length");
        }
        self.get_video_length_in_seconds()?;
        if !settings.quiet {
            eprintln!(
                "Extracting sound from video. This should take about {} seconds.",
                (self.length_seconds.unwrap() / 60.0).round()
            );
        }
        let sound = self.extract_mp3_stream_from_video()?;
        if !settings.quiet {
            eprintln!("Guessing which parts of video are silent or loud. This should take about {} seconds.", (self.length_seconds.unwrap() / 120.0).round());
        }
        self.recognize_silent_and_loud_frames(sound, settings);
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
        let duration_command = Command::new("ffprobe")
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
            }
            _ => {}
        }
        let mut output = duration_command.stdout.unwrap();
        let mut output_buffer: Vec<u8> = Vec::new();
        output.read_to_end(&mut output_buffer)?;
        let output_as_string = String::from_utf8(output_buffer)?;
        self.length_seconds = Some(output_as_string.trim().parse()?);
        Ok(())
    }

    /// Use ffmpeg to extract mp3 stream from video.
    /// This only uses one thread, but achieves roughly 60x
    /// speed of playback on my intel core i5 g8.
    ///
    /// TODO: Return as stream, so we can
    /// process the audio while ffmpeg is working.
    fn extract_mp3_stream_from_video(&self) -> Result<Vec<u8>, VideoProcessingError> {
        let parameter_source = match &self.source {
            VideoSource::FilePath(s) => s,
            VideoSource::StdinStream(_) => "-",
        };
        let parameter_stdin = match &self.source {
            VideoSource::FilePath(_) => Stdio::null(),
            VideoSource::StdinStream(_) => Stdio::piped(),
        };
        let convert_to_mp3_command = Command::new("ffmpeg")
            .arg("-i")
            .arg(parameter_source)
            .arg("-vn")
            .arg("-f")
            .arg("mp3")
            .arg("-")
            .stdout(Stdio::piped())
            .stdin(parameter_stdin)
            .stderr(Stdio::null())
            .spawn()?;
        match &self.source {
            VideoSource::StdinStream(stream) => {
                // Send stdin contents to the process
                let mut command_stdin = convert_to_mp3_command.stdin.unwrap();
                let mut writer = BufWriter::new(&mut command_stdin);
                writer.write_all(stream)?;
            }
            _ => {}
        }
        let mut output = convert_to_mp3_command.stdout.unwrap();
        let mut output_buffer: Vec<u8> = Vec::new();
        output.read_to_end(&mut output_buffer)?;
        Ok(output_buffer)
    }

    fn recognize_silent_and_loud_frames(&mut self, sound: Vec<u8>, settings: &Cli) {
        let mut silent_frames: Vec<bool>;

        let mut sound_decoder = Decoder::new(&sound[..]);
        let mut sound_averages: Vec<usize> = Vec::new();
        let mut sound_max: usize = 0;
        let mut all_frames_data: Vec<Vec<i16>> = Vec::new();

        // Save all frames data
        loop {
            match sound_decoder.next_frame() {
                Ok(frame) => {
                    all_frames_data.push(frame.data);
                }
                Err(Error::Eof) => break,
                Err(e) => panic!(e), // TODO: Don't pacic
            };
        }
        // Go through the frames data
        // Calculate average for current frame,
        // and record maximum average.
        for frame in &all_frames_data {
            let avg = frame.iter().fold(0, |sum, val| sum + val.abs() as usize) / frame.len();
            if sound_max < avg {
                sound_max = avg;
            }
            sound_averages.push(avg);
        }
        let silent_level = sound_max as f32 * settings.silence_threshold;
        silent_frames = sound_averages
            .iter()
            .map(|avg| avg < &(silent_level as usize))
            .collect();

        // Smooth silent frames
        // TODO: We should be able to make this faster
        for _ in 0..settings.frame_margin {
            let mut frames_to_be_loud: Vec<bool> = Vec::with_capacity(silent_frames.len());
            frames_to_be_loud.push(false);
            if silent_frames.len() > 0 {
                for i in 1..silent_frames.len() - 1 {
                    if silent_frames[i] == true
                        && (silent_frames[i - 1] == false || silent_frames[i + 1] == false)
                    {
                        frames_to_be_loud.push(true);
                    } else {
                        frames_to_be_loud.push(false);
                    }
                }
            }
            for i in 0..frames_to_be_loud.len() {
                if frames_to_be_loud[i] == true {
                    silent_frames[i] = false;
                }
            }
        }
        self.silent_frames = Some(silent_frames);
    }
}

pub enum VideoProcessingError {
    ExternalIOError(std::io::Error),
    FaieldToReadOutput(std::string::FromUtf8Error),
    BadOutputFromFFprobe(std::num::ParseFloatError),
    FailedToExtractAudio,
}
impl Display for VideoProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason: String = match self {
            VideoProcessingError::ExternalIOError(e) => {
                format!("Failed to spawn process of ffmpeg/ffprobe: {}", e)
            }
            VideoProcessingError::FaieldToReadOutput(e) => format!(
                "Failed to read output of ffmpeg/ffprobe process, output was not valid utf-8: {}",
                e
            ),
            VideoProcessingError::BadOutputFromFFprobe(e) => {
                format!("Bad output from ffprobe, failed to parse to f32: {}", e)
            }
            VideoProcessingError::FailedToExtractAudio => {
                format!("Failed to extract mp3 audio stream for video.")
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

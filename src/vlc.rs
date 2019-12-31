use crate::video_processing::{Video, VideoSource};

use guid_create::GUID;
use std::fmt::{self, Display};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

pub struct VlcController {
    filename: String,
    video_frames: Vec<bool>,
    video_seconds: f32,
    speedup_loud: f32,
    speedup_silent: f32,
}
impl VlcController {
    /// Save info into vlc controller.
    ///
    /// If the video was sent to stdin, save it to named pipe.
    /// This will fail if `mkfifo` doesn't exist.
    pub fn new(video_info: Video, speedup_loud: f32, speedup_silent: f32) -> Result<VlcController, VlcControllerError> {
        let filename: String;
        if let VideoSource::StdinStream(stream) = video_info.source {
            // If user passed video as stream into stdin, save it to
            // temporary fifo file
            filename = GUID::rand().to_string();
            Command::new("mkfifo").arg(&filename).output()?;
            // Write to fifo in new thread
            // TODO: Get rid of panic inside the thread?
            let fifo_filename_cloned = filename.clone();
            thread::spawn(move || {
                let mut fifo = OpenOptions::new()
                    .write(true)
                    .open(fifo_filename_cloned)
                    .unwrap();
                fifo.write_all(&stream[..]).unwrap();
            });
        } else {
            if let VideoSource::FilePath(file) = video_info.source {
                filename = file;
            } else {
                panic!("Fatal error: missing video information for VLC process. Open bug report at >>https://github.com/soptikha2/video-summarizer<<");
            }
        }

        if video_info.silent_frames.is_none() || video_info.length_seconds.is_none() {
            return Err(VlcControllerError::NotEnoughInfo);
        }

        Ok(VlcController {
            filename,
            video_frames: video_info.silent_frames.unwrap(),
            video_seconds: video_info.length_seconds.unwrap(),
            speedup_loud,
            speedup_silent,
        })
    }
    /// Start VLC and control it based on values received when this controller was created.
    ///
    /// This doesn't return until the VLC quits, so it might be good idea to
    /// run this function in a separate thread.
    pub fn start(self) -> Result<(), VlcControllerError> {
        // Start VLC with parameter -Irc
        let vlc_command = Command::new("vlc")
            .arg("-Irc")
            .arg(&self.filename)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        // Spawn new thread that will just read/write to/from vlc process forever
        let vlc_thread = thread::spawn(move || {
            let vlc_controller = self;
            let mut vlc_stdin = vlc_command.stdin.unwrap();
            let mut vlc_stdout = vlc_command.stdout.unwrap();
            loop {
                // Repeat this every 200ms...
                thread::sleep(Duration::from_millis(200));
                // Ask vlc if the video is playing right now.
                let mut vlc_buffer: Vec<u8> = Vec::new();
                vlc_stdin.write("is_playing".as_bytes()).unwrap();
                vlc_stdout.read_to_end(&mut vlc_buffer).unwrap();
                let is_playing = VlcController::parse_vlc_response_to_usize(String::from_utf8(vlc_buffer).unwrap()).unwrap();
                if is_playing == 0 {
                    // Wait until user starts playing the video
                    continue;
                }
                // Ask vlc for time
                let mut vlc_buffer: Vec<u8> = Vec::new();
                vlc_stdin.write("get_time".as_bytes()).unwrap();
                vlc_stdout.read_to_end(&mut vlc_buffer).unwrap();
                let time_seconds = VlcController::parse_vlc_response_to_usize(String::from_utf8(vlc_buffer).unwrap()).unwrap();
                let speedup = vlc_controller.is_silent_in_current_second(time_seconds);
                // Set speedup rate
                vlc_stdin.write(format!("rate {}", speedup).as_bytes()).unwrap();
            }
        });

        // Wait for it to finish.
        // The thread will end when vlc process exits.
        vlc_thread.join();

        Ok(())
    }

    /// Take vlc response line which looks like this:
    /// > 1
    /// and try to parse the number that is on the line.
    fn parse_vlc_response_to_usize(line: String) -> Result<usize, std::num::ParseIntError> {
        line.replace(">", "").trim().parse()
    }

    fn is_silent_in_current_second(&self, second: usize) -> bool {
        
        unimplemented!();
    }

    fn get_speedup_for_current_second(&self, second: usize) -> f32 {

        unimplemented!();
    }
}

#[derive(Debug)]
pub enum VlcControllerError {
    ExternalCommandError(std::io::Error),
    NotEnoughInfo,
}
impl From<std::io::Error> for VlcControllerError {
    fn from(e: std::io::Error) -> Self {
        VlcControllerError::ExternalCommandError(e)
    }
}
impl Display for VlcControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason: String = match self {
                VlcControllerError::ExternalCommandError(e) => {
                    format!("Failed to spawn process of mkfifo/vlc: {}", e)
                }
                VlcControllerError::NotEnoughInfo => format!(
                    "We don't have enough info about video to start playing it. Use `video.analyze_sound().`"
                ),
            };
        write!(f, "An error occured while processing video: {}", reason)
    }
}

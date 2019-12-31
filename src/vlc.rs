use crate::video_processing::{Video, VideoSource};

use guid_create::GUID;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::*;
use std::process::{Command, Stdio};
use std::thread;

pub struct VlcController {
    filename: String,
}
impl VlcController {
    /// Save info into vlc controller.
    ///
    /// If the video was sent to stdin, save it to named pipe.
    /// This will fail if `mkfifo` doesn't exist.
    pub fn new(video_info: Video) -> Result<VlcController, VlcControllerError> {
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

        Ok(VlcController { filename })
    }
    /// Start VLC and control it based on values received when this controller was created.
    ///
    /// This doesn't return until the VLC quits, so it might be good idea to
    /// run this function in a separate thread.
    pub fn start(&self) -> Result<(), VlcControllerError> {
        // Start VLC with parameter -Irc
        // let duration_command = Command::new("vlc")
        //     .arg("-Irc")
        //     .arg(parameter_source)
        //     .stdout(Stdio::piped())
        //     .stdin(parameter_stdin)
        //     .stderr(Stdio::null())
        //     .spawn()?;
        unimplemented!();
    }
}

pub enum VlcControllerError {
    ExternalCommandError(std::io::Error),
}
impl From<std::io::Error> for VlcControllerError {
    fn from(e: std::io::Error) -> Self {
        VlcControllerError::ExternalCommandError(e)
    }
}

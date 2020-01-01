use crate::video_processing::{Video, VideoSource};

use guid_create::GUID;
use std::fmt::{self, Display};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

extern crate rand;

extern crate telnet;
use telnet::{Telnet, TelnetEvent};

/// There are multiple ways to control VLC.
/// 
/// The simplest one seems to be to pass `-I rc`
/// (remote control) switch, which allows user
/// to send commands and receive replies via stdin/out.
/// 
/// Unfortunatelly, this doesn't work when used programatically,
/// and trust me, I tried.
/// 
/// Thankfully, VLC allows one to specify telnet connection
/// like this: `vlc -I telnet --telnet-password mypassw0rd!`,
/// which hosts telnet connection on port 4212 by default.
/// 
/// We can easily access VLC via telnet.
pub struct VlcController {
    filename: String,
    video_frames: Vec<bool>,
    video_seconds: f32,
    speedup_loud: f32,
    speedup_silent: f32,
    telnet_connection: Option<Telnet>,
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
            telnet_connection: None,
        })
    }
    /// Start VLC and control it based on values received when this controller was created.
    ///
    /// This doesn't return until the VLC quits, so it might be good idea to
    /// run this function in a separate thread.
    pub fn start(mut self) -> Result<(), VlcControllerError> {
        let telnet_password: String = self.generate_telnet_password(128);

        // Start VLC with parameter -Irc
        let vlc_command = Command::new("vlc")
            .arg("-I telnet")
            .arg("--telnet-password")
            .arg(&telnet_password)
            .arg(&self.filename)
            .stdout(Stdio::null())
            .stdin(Stdio::null()) //todo pipe from stdin
            .spawn()?;

        // Wait a bit
        // TODO: sane solution
        thread::sleep(Duration::from_millis(200));
        
            // Initialize telnet
            self.initialize_vlc_telnet(telnet_password);
            thread::sleep(Duration::from_millis(500));
            // let mut vlc_stdin = vlc_command.stdin.unwrap();
            // let mut vlc_stdout = vlc_command.stdout.unwrap();
            loop {
               // Repeat this every 200ms...
                thread::sleep(Duration::from_millis(200));
                // Ask vlc if the video is playing right now.
                self.send_to_vlc_via_telnet("is_playing");
                thread::sleep(Duration::from_millis(200));
                let is_playing = VlcController::parse_vlc_response_to_usize(self.receive_response_from_telnet()).unwrap();
                if is_playing == 0 {
                    // Wait until user starts playing the video
                    continue;
                }
                // Ask vlc for time
                let time_seconds = self.get_current_time().unwrap();
                let speedup = self.get_speedup_for_current_second(time_seconds);
                // Set speedup rate
                self.send_to_vlc_via_telnet(&format!("rate {}", speedup));
                eprintln!("Set rate to {}", speedup);
            }

        // Wait for it to finish.
        // The thread will end when vlc process exits.
        //vlc_thread.join();

        // TODO: unwrap -> result, when reading fails, keep reading; there are more rows.

        Ok(())
    }

    fn initialize_vlc_telnet(&mut self, password: String) {
        self.telnet_connection = Some(Telnet::connect(("localhost", 4212), 256).unwrap());
        // Send password
        self.telnet_connection.as_mut().unwrap().write(format!("{}\n", password).as_bytes()).unwrap();
        // Ignore welcoming messages
        for _ in 0..2 {
            eprintln!("Ignoring: {}", self.receive_response_from_telnet());
        }
    }

    fn send_to_vlc_via_telnet(&mut self, command: &str) {
        self.telnet_connection.as_mut().unwrap().write(format!("{}\n", command).as_bytes()).unwrap();
    }

    /// Blocking read from telnet, unspecified amount of bytes.
    /// This might not read to end, see `receive_response_from_telnet_until_found`
    fn receive_response_from_telnet(&mut self) -> String {
        let event = self.telnet_connection.as_mut().unwrap().read().unwrap();

        match event {
            TelnetEvent::Data(buffer) => {
                return String::from_utf8((*buffer).to_vec()).unwrap();
            }
            _ => { return self.receive_response_from_telnet(); }
        }
    }

    /// Keep blocking-reading from telnet until everything specified in required_contents was read.
    fn receive_response_from_telnet_until_found(&mut self, required_contents: Vec<&str>) -> String {
        let mut seen: Vec<bool> = required_contents.iter().map(|_| false).collect();

        let mut response = String::new();
        while seen.iter().any(|s| *s == false) {
            eprintln!("Still waiting for {:?}", required_contents.iter().zip(&seen).filter(|(_, found)| !*found).map(|(contents, _)| contents));

            let newest_response = self.receive_response_from_telnet();
            for i in 0..required_contents.len() {
                if seen[i] {
                    continue;
                }
                if newest_response.contains(required_contents[i]) {
                    seen[i] = true;
                }
            }
            response.push_str(&newest_response);
        }
        response
    }

    /// Take vlc response line which looks like this:
    /// > 1
    /// and try to parse the number that is on the line.
    fn parse_vlc_response_to_usize(line: String) -> Option<usize> {
        // Go for each whitespace, return first number found
        line.split_whitespace().filter_map(|s| s.parse::<usize>().ok()).next()
    }

    /// Execute multiple requests to get precise current time.
    /// First of all, we send "stats" and get numbers from lines:
    /// ```
    /// | frames displayed :     2704
    /// | frames lost :          0
    /// ```
    /// 
    /// Then, we look at video framerate using "info" command:
    /// ```
    /// | Frame rate: 23.976024
    /// ```
    /// 
    /// Using this, we can calculate precise current time.
    /// 
    /// If any of this is missing, return none.
    fn get_current_time(&mut self) -> Option<f32> {
        // Send stats and info commands
        self.send_to_vlc_via_telnet("stats");
        self.send_to_vlc_via_telnet("info");
        let mut response: String = self.receive_response_from_telnet_until_found(vec!["frames displayed", "frames lost", "Frame rate"]);
        eprintln!("Time output:\n{}\n--END_TIME_OUTPUT--", response);
        let frames_displayed_line: &str = response.lines().filter(|line| line.contains("frames displayed")).next()?;
        let frames_displayed = frames_displayed_line.replace(" ", "").split(":").last()?.parse::<usize>().ok()?;
        let frames_lost_line: &str = response.lines().filter(|line| line.contains("frames lost")).next()?;
        let frames_lost = frames_lost_line.replace(" ", "").split(":").last()?.parse::<usize>().ok()?;
        let frame_rate_line: &str = response.lines().filter(|line| line.contains("Frame rate")).next()?;
        let fps = frame_rate_line.replace(" ", "").split(":").last()?.parse::<f32>().ok()?;

        Some((frames_displayed + frames_lost) as f32 / fps)
    }

    fn is_silent_in_current_second(&self, second: f32) -> bool {
        let current_index = second * (self.video_frames.len() as f32 / self.video_seconds);
        self.video_frames[current_index.floor() as usize]
    }

    fn get_speedup_for_current_second(&self, second: f32) -> f32 {
        match self.is_silent_in_current_second(second) {
            true => {
                self.speedup_silent
            }
            false => {
                self.speedup_loud
            }
        }
    }

    /// Generate random password for telnet connection
    /// that we use to send commands to VLC.
    /// 
    /// This is not cryptographically secure, but
    /// it doesn't really need to be.
    fn generate_telnet_password(&self, length: usize) -> String {
        if length < 4 {
            panic!("Invalid telnet password length.");
        }

        let allowed_characters: Vec<char> = "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM;',./?><|\":}{P})(*&^%$#@!)".chars().collect();

        let mut password = String::with_capacity(length);
        for i in 0..length {
            // We don't need cryptographical security
            // If anyone wants to have it, please submit a pull request
            let rnd = rand::random::<usize>() % allowed_characters.len();
            password.push(allowed_characters[rnd]);
        }
        password
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

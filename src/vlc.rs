use crate::video_processing::{Video, VideoSource};

use std::fmt::{self, Display};
use std::io::{BufWriter, Write};
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
    source: VideoSource,
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
        if video_info.silent_frames.is_none() || video_info.length_seconds.is_none() {
            return Err(VlcControllerError::NotEnoughInfo);
        }

        Ok(VlcController {
            source: video_info.source,
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

        let parameter_source = match &self.source {
            VideoSource::FilePath(s) => s,
            VideoSource::StdinStream(_) => "-",
        };
        let parameter_stdin = match &self.source {
            VideoSource::FilePath(_) => Stdio::null(),
            VideoSource::StdinStream(_) => Stdio::piped(),
        };
        // Start VLC with parameter -Irc
        let vlc_command = Command::new("vlc")
            .arg("-I telnet")
            .arg("--telnet-password")
            .arg(&telnet_password)
            .arg(parameter_source)
            .stdin(parameter_stdin)
            .stdout(Stdio::null())
            .spawn()?;
        match &self.source {
            VideoSource::StdinStream(stream) => {
                // Send stdin contents to the process
                let mut command_stdin = vlc_command.stdin.unwrap();
                let mut writer = BufWriter::new(&mut command_stdin);
                writer.write_all(stream)?;
            }
            _ => {}
        }

        // Wait a bit
        // TODO: sane solution
        thread::sleep(Duration::from_millis(200));
        
            // Initialize telnet
            self.initialize_vlc_telnet(telnet_password)?;
            thread::sleep(Duration::from_millis(500));
            loop {
               // Repeat this every 200ms...
                thread::sleep(Duration::from_millis(200));
                // Ask vlc if the video is playing right now.
                self.send_to_vlc_via_telnet("is_playing")?;
                thread::sleep(Duration::from_millis(200));
                let is_playing = VlcController::parse_vlc_response_to_usize(self.receive_response_from_telnet()?);
                if is_playing.is_none() { break; }
                if is_playing.unwrap() == 0 {
                    // Wait until user starts playing the video
                    continue;
                }
                // Ask vlc for time
                let time_seconds = self.get_current_time();
                if time_seconds.is_none() { break; }
                let speedup = self.get_speedup_for_current_second(time_seconds.unwrap());
                // Set speedup rate
                self.send_to_vlc_via_telnet(&format!("rate {}", speedup))?;
            }

        // Wait for it to finish.
        // The thread will end when vlc process exits.

        Ok(())
    }

    fn initialize_vlc_telnet(&mut self, password: String) -> Result<(), VlcControllerError> {
        self.telnet_connection = Some(Telnet::connect(("localhost", 4212), 256)?);
        // Send password
        self.telnet_connection.as_mut().unwrap().write(format!("{}\n", password).as_bytes())?;
        // Ignore welcoming messages
        for _ in 0..2 {
            self.receive_response_from_telnet()?;
        }

        Ok(())
    }

    fn send_to_vlc_via_telnet(&mut self, command: &str) -> Result<(), VlcControllerError> {
        self.telnet_connection.as_mut().unwrap().write(format!("{}\n", command).as_bytes())?;
        Ok(())
    }

    /// Blocking read from telnet, unspecified amount of bytes.
    /// This might not read to end, see `receive_response_from_telnet_until_found`
    fn receive_response_from_telnet(&mut self) -> Result<String, VlcControllerError> {
        let event = self.telnet_connection.as_mut().unwrap().read()?;

        match event {
            TelnetEvent::Data(buffer) => {
                return Ok(String::from_utf8((*buffer).to_vec())?);
            }
            _ => { return self.receive_response_from_telnet(); }
        }
    }

    /// Keep blocking-reading from telnet until everything specified in required_contents was read.
    fn receive_response_from_telnet_until_found(&mut self, required_contents: Vec<&str>) -> Result<String, VlcControllerError> {
        let mut seen: Vec<bool> = required_contents.iter().map(|_| false).collect();

        let mut response = String::new();
        while seen.iter().any(|s| *s == false) {
            let newest_response = self.receive_response_from_telnet()?;
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
        Ok(response)
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
        self.send_to_vlc_via_telnet("stats").ok()?;
        self.send_to_vlc_via_telnet("info").ok()?;
        let response: String = self.receive_response_from_telnet_until_found(vec!["frames displayed", "frames lost", "Frame rate"]).ok()?;
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
        for _ in 0..length {
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
    CommunicationErrorWithVlc(std::string::FromUtf8Error),
}
impl From<std::io::Error> for VlcControllerError {
    fn from(e: std::io::Error) -> Self {
        VlcControllerError::ExternalCommandError(e)
    }
}
impl From<std::string::FromUtf8Error> for VlcControllerError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        VlcControllerError::CommunicationErrorWithVlc(err)
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
                VlcControllerError::CommunicationErrorWithVlc(e) => format!(
                    "Something went wrong while communicating with VLC: {}", e
                )
            };
        write!(f, "An error occured while processing video: {}", reason)
    }
}

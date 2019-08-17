use minimp3::{Decoder, Error};
use regex::Regex;
use structopt::StructOpt;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    let mut args: Cli = Cli::from_args();
    // Set output filename if not set by user
    if args
        .output
        .to_str()
        .expect("Failed to get output filename.")
        .len()
        == 0
    {
        if args.input.to_str().expect("Failed to get input filename.") == "-" {
            // TODO: Fix sound extraction
            eprintln!("Piping video in isn't supported yet. Sorry!");
            return;
            args.output = PathBuf::from("-");
        } else {
            args.output = PathBuf::from(format!(
                "{}.new.{}",
                args.input
                    .file_stem()
                    .expect("Failed to get file stem from input file path.")
                    .to_str()
                    .unwrap(),
                args.input
                    .extension()
                    .unwrap_or_else(|| OsStr::new(""))
                    .to_str()
                    .unwrap()
            ));
        }
    }

    // Get general video metadata
    let video_metadata: VideoMetadata = get_video_metadata(args.input.to_str().unwrap());

    let mut silent_frames: Vec<bool>;
    // Detect silent frames
    {
        // Extract sound from video
        let mut sound = Command::new("ffmpeg")
            .arg("-i")
            .arg(args.input.to_str().unwrap())
            .arg("-vn")
            .arg("-f")
            .arg("mp3")
            .arg("-")
            .stdout(Stdio::piped())
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .expect("Failed to spawn sound extract process.");
        let output = sound.stdout;
        let mut sound_decoder = Decoder::new(&output[..]);
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
                Err(e) => panic!(e),
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
        let silent_level = sound_max as f32 * args.silence_threshold;
        silent_frames = sound_averages
            .iter()
            .map(|avg| avg < &(silent_level as usize))
            .collect();

        // Smooth silent frames
        // TODO: Not like it matters, but this can be done way faster
        for _ in 0..args.frame_margin {
            let mut frames_to_be_loud: Vec<bool> = Vec::with_capacity(silent_frames.len());
            frames_to_be_loud.push(false);
            for i in 1..silent_frames.len() - 1 {
                if silent_frames[i] == true
                    && (silent_frames[i - 1] == false || silent_frames[i + 1] == false)
                {
                    frames_to_be_loud.push(true);
                } else {
                    frames_to_be_loud.push(false);
                }
            }
            for i in 0..frames_to_be_loud.len() {
                if frames_to_be_loud[i] == true {
                    silent_frames[i] = false;
                }
            }
        }
    }

    // Compute speedup ranges
    // Note: speedup ranges contain frames,
    // but those are AUDIO frames! Audio frames
    // might not match video frames.
    let mut frames_speedup: Vec<SpeedupRange> = Vec::new();
    {
        let mut current_speedup = SpeedupRange::new(
            0,
            0,
            if silent_frames[0] {
                args.speed_loud
            } else {
                args.speed_silent
            },
        );
        let mut current_speedup_loudness: bool = silent_frames[0];
        for i in 1..silent_frames.len() {
            if silent_frames[i] == current_speedup_loudness {
                continue;
            } else {
                current_speedup.frame_to = i;
                frames_speedup.push(current_speedup);
                current_speedup = SpeedupRange::new(
                    i,
                    i,
                    if silent_frames[i] {
                        args.speed_loud
                    } else {
                        args.speed_silent
                    },
                );
                current_speedup_loudness = silent_frames[i];
            }
        }
        current_speedup.frame_to = silent_frames.len() - 1;
        frames_speedup.push(current_speedup);
    }
    // Figure out where to cut video
    {
        // Map speedup ranges to video frames
        let last_audio_frame = frames_speedup.last().unwrap().frame_to;
        let last_video_frame = video_metadata.total_frames;
        let rate: f32 = last_video_frame as f32 / last_audio_frame as f32;
    }
    println!("{:#?}", frames_speedup);
}

fn get_video_metadata(filename: &str) -> VideoMetadata {
    let regex_video_duration: Regex = Regex::new(r"DURATION\s+: [0-9:.]+").unwrap();
    let regex_fps: Regex = Regex::new(r"[0-9]+ fps").unwrap();
    let regex_frames: Regex = Regex::new(r"frame=\s+[0-9]+").unwrap();

    let mut metadata_command = Command::new("ffmpeg")
        .args(&["-i", filename, "-f", "null", "-"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()
        .expect("Failed to spawn video metadata process");

    let output_string = String::from_utf8(metadata_command.stderr).unwrap();

    let duration_position = regex_video_duration
        .find(&output_string)
        .expect("Failed to extract video duration.");
    let duration_in_string = duration_position.as_str().split(":").collect::<Vec<&str>>();
    let duration_seconds: f32 = duration_in_string.last().unwrap().parse().expect(&format!(
        "Failed to parse video duration from {}",
        duration_in_string.last().unwrap()
    ));

    let fps_position = regex_fps
        .find(&output_string)
        .expect("Failed to extract fps.");
    let fps_in_string = fps_position
        .as_str()
        .split_whitespace()
        .collect::<Vec<&str>>();
    let fps: f32 = fps_in_string.first().unwrap().parse().expect(&format!(
        "Failed to parse fps from {}",
        fps_in_string.first().unwrap()
    ));

    let total_frames_position = regex_frames
        .find_iter(&output_string)
        .last()
        .expect("Failed to extract total video frames.");
    let total_frames_in_string = total_frames_position
        .as_str()
        .split_whitespace()
        .collect::<Vec<&str>>();
    let total_frames: usize = total_frames_in_string
        .last()
        .unwrap()
        .parse()
        .expect(&format!(
            "Failed to parse total video frames from {}",
            total_frames_in_string.last().unwrap()
        ));

    VideoMetadata {
        duration_seconds,
        fps,
        total_frames,
    }
}

#[derive(StructOpt)]
#[structopt(
    name = "Video Summarizer",
    about = "Take a video, and change it's speed, depending on silent and loud parts.",
    rename_all = "kebab-case"
)]
struct Cli {
    /// Source video
    ///
    /// Path to source video. Video must be
    /// parsable by FFMPEG.
    #[structopt(parse(from_os_str))]
    input: std::path::PathBuf,
    /// Output file
    ///
    /// This is by default "old_filename.new.extension"
    #[structopt(parse(from_os_str), short = "o", default_value = "")]
    output: std::path::PathBuf,
    /// Video speed when loud sound is detected.
    #[structopt(long = "speed-loud", short = "l", default_value = "1")]
    speed_loud: f32,
    /// Video speed when no loud sound was detected.
    #[structopt(long = "speed-silent", short = "s", default_value = "2")]
    speed_silent: f32,
    /// Threshold of silence. When sound gets under this threshold,
    /// current frame will be considered as silent.
    ///
    /// If it sounds as if the speech is cut out right at start/end,
    /// consider editing "frame-margin" option first.
    #[structopt(long = "silence-threshold", default_value = "0.02")]
    silence_threshold: f32,
    /// Number of frames before/after loud frames to be considered
    /// loud as well, even if they actually aren't.
    ///
    /// Use this settings if beginning/end of sentences
    /// get cut out/sped up as they are considered silent.
    #[structopt(long = "frame_margin", default_value = "2")]
    frame_margin: usize,
}

#[derive(Debug)]
struct SpeedupRange {
    frame_from: usize,
    frame_to: usize,
    speedup_rate: f32,
}
impl SpeedupRange {
    pub fn new(frame_from: usize, frame_to: usize, speedup_rate: f32) -> SpeedupRange {
        SpeedupRange {
            frame_from,
            frame_to,
            speedup_rate,
        }
    }
}

struct VideoMetadata {
    fps: f32,
    duration_seconds: f32,
    total_frames: usize,
}

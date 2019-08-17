use guid_create::GUID;
use minimp3::{Decoder, Error};
use regex::Regex;
use structopt::StructOpt;

use std::fs;
use std::path::{Path, PathBuf};
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
                "{}.new.mpeg",
                args.input
                    .file_name()
                    .expect("Failed to get file stem from input file path.")
                    .to_str()
                    .unwrap()
            ));
        }
    }
    // If output file exists, delete it
    if args.output.to_str().expect("Failed to get output") != "-" {
        if args.output.exists() {
            fs::remove_file(&args.output).expect("Failed to delete existing output file.");
        }
    }

    // Get general video metadata
    let video_metadata: VideoMetadata = get_video_metadata(args.input.to_str().unwrap());

    let mut silent_frames: Vec<bool>;
    // Detect silent frames
    {
        // Extract sound from video
        let sound = Command::new("ffmpeg")
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
    let mut audio_frames_speedup: Vec<SpeedupRange> = Vec::new();
    {
        let mut current_speedup = SpeedupRange::new(
            0,
            0,
            if silent_frames[0] {
                args.speed_silent
            } else {
                args.speed_loud
            },
        );
        let mut current_speedup_loudness: bool = silent_frames[0];
        for i in 1..silent_frames.len() {
            if silent_frames[i] == current_speedup_loudness {
                continue;
            } else {
                current_speedup.frame_to = i;
                audio_frames_speedup.push(current_speedup);
                current_speedup = SpeedupRange::new(
                    i,
                    i,
                    if silent_frames[i] {
                        args.speed_silent
                    } else {
                        args.speed_loud
                    },
                );
                current_speedup_loudness = silent_frames[i];
            }
        }
        current_speedup.frame_to = silent_frames.len() - 1;
        audio_frames_speedup.push(current_speedup);
    }
    let video_frames_speedup: Vec<SpeedupRange>;
    // Figure out where to cut video
    {
        // Map speedup ranges to video frames
        let last_audio_frame = audio_frames_speedup.last().unwrap().frame_to;
        let last_video_frame = video_metadata.total_frames;
        let rate: f32 = last_video_frame as f32 / last_audio_frame as f32;
        video_frames_speedup = audio_frames_speedup
            .iter()
            .map(|range| {
                SpeedupRange::new(
                    (range.frame_from as f32 * rate) as usize,
                    (range.frame_to as f32 * rate) as usize,
                    range.speedup_rate,
                )
            })
            .collect();
    }
    // Do the splitting, speed-uping, etc
    {
        // Create temporary directory where we will store everything.
        let tempdir_path = std::env::temp_dir().join(GUID::rand().to_string());
        fs::DirBuilder::new()
            .create(&tempdir_path)
            .expect("Failed to create tmp directory.");

        // Split and speedup videos, get these part names in order.
        let video_part_paths = video_frames_speedup.iter().map(|frame| {
            speedup_video_part(
                args.input.to_str().unwrap(),
                &frame,
                &video_metadata,
                &tempdir_path,
            )
        });

        // Create result mpeg file, and add concat everything
        concatenate_video_to_file(
            &video_part_paths
                .filter(|p| p.is_some())
                .map(|p| String::from(p.unwrap().to_str().unwrap()))
                .collect::<Vec<String>>()
                .join("|"),
            args.output,
        );

        fs::remove_dir_all(&tempdir_path).expect("Failed to remove tmp directory.");
    }
}

fn get_video_metadata(filename: &str) -> VideoMetadata {
    let regex_video_duration: Regex = Regex::new(r"DURATION\s+: [0-9:.]+").unwrap();
    let regex_fps: Regex = Regex::new(r"[0-9]+ fps").unwrap();
    let regex_frames: Regex = Regex::new(r"frame=\s+[0-9]+").unwrap();

    let metadata_command = Command::new("ffmpeg")
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

/// Take input video, separate one part from it,
/// speed it up and return path to the sped up video.
///
/// If speed is lower than 0.5, panic.
/// If speed is higher than 100, return `None`.
fn speedup_video_part(
    input_path: &str,
    range: &SpeedupRange,
    metadata: &VideoMetadata,
    tempdir_path: &std::path::Path,
) -> Option<PathBuf> {
    if range.speedup_rate < 0.5 {
        panic!("Fatal error: speed rate is lower than 0.5.");
    }
    if range.speedup_rate > 100.0 {
        return None;
    }

    let cut_video_filename = GUID::rand().to_string();
    let speedup_video_filename = GUID::rand().to_string();
    let cut_video_path = tempdir_path.join(Path::new(&cut_video_filename));
    let speedup_video_path = tempdir_path.join(Path::new(&speedup_video_filename));

    let seconds_to_start_cut: f32 = range.frame_from as f32 / metadata.fps;
    let inverted_speedup_rate = 1.0 / range.speedup_rate;

    // Cut video
    let mut cut_command = Command::new("ffmpeg")
        .args(&[
            "-ss",
            &format!("{}", seconds_to_start_cut),
            "-i",
            input_path,
            "-frames:v",
            &format!("{}", range.frame_to - range.frame_from),
            "-f",
            "mpeg",
            cut_video_path.to_str().unwrap(),
        ])
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to cut input video.");
    cut_command.wait().unwrap();

    // Speedup video
    let mut speedup_command = Command::new("ffmpeg")
        .args(&[
            "-i",
            cut_video_path.to_str().unwrap(),
            "-filter_complex",
            &format!(
                "[0:v]setpts={}*PTS[v];[0:a]atempo={}[a]",
                inverted_speedup_rate, range.speedup_rate
            ),
            "-map",
            "[v]",
            "-map",
            "[a]",
            "-f",
            "mpeg",
            speedup_video_path.to_str().unwrap(),
        ])
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to spawn speedup video process.");

    speedup_command.wait().unwrap();

    Some(speedup_video_path)
}

fn concatenate_video_to_file(filenames_delimited_by_pipe: &str, output_path: PathBuf) {
    Command::new("ffmpeg")
        .args(&[
            "-i",
            &format!("concat:{}", filenames_delimited_by_pipe),
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to run video concatenate process")
        .wait()
        .expect("Failed to concatenate video files.");
}

#[derive(StructOpt)]
#[structopt(
    name = "Video Summarizer",
    about = "Take a video, and change it's speed, depending on silent and loud parts. New video will be in .mpeg format for performance purposes.",
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
    /// This is by default "old_filename.new.mpeg".
    /// This will always be in .mpeg format, as it
    /// would take about twice as much time to
    /// return video in specified format.
    #[structopt(parse(from_os_str), short = "o", default_value = "")]
    output: std::path::PathBuf,
    /// Video speed when loud sound is detected.
    ///
    /// This has to be at least 0.5.
    /// If this is larger than 100, loud parts of
    /// the video will be dropped completely.
    #[structopt(long = "speed-loud", short = "l", default_value = "1")]
    speed_loud: f32,
    /// Video speed when no loud sound was detected.
    ///
    /// This has to be at least 0.5.
    /// If this is larger than 100, silent parts
    /// of the video will be dropped completely.
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

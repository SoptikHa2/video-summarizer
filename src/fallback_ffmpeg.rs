/// This is fallback solution for video summarizer.
/// 
/// It uses ffmpeg, no concurrency, and man, the spaghetti.
/// 
/// This section is no longer mantained. It might break or be removed
/// at any time and won't be fixed.
pub mod fallback_ffmpeg {
    // TODO: Remove file GUID creation for fast option, use something predictable instead.
    // Sometimes GUID filenames might clash, even if it's very unlikely to happen.
    // Even with approx. 45min long video, the chance of clash would be just something like
    // 5 : 5,316,911,983,139,663,491,615,228,241,121,400
    use guid_create::GUID;
    use minimp3::{Decoder, Error};

    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use crate::cli::Cli;

    pub fn solve(mut args: Cli) {
        if args.output.to_str().expect("Failed to get output filename").len() != 0 {
            // User specified output filename.
            // Fallback to ffmpeg.
        }


        // Set output filename if not set by user
        if args
            .output
            .to_str()
            .expect("Failed to get output filename.")
            .len()
            == 0
        {
            if args.input.to_str().expect("Failed to get input filename.") == "-" {
                eprintln!("Piping video in isn't supported yet. Sorry!");
                return;
            } else {
                args.output = PathBuf::from(format!(
                    "{}.new.{}",
                    args.input
                        .file_stem()
                        .expect("Failed to get file stem from input file path.")
                        .to_str()
                        .unwrap(),
                    if args.fast {
                        "mpeg"
                    } else {
                        args.input
                            .file_name()
                            .expect("Failed to get file name from input file path.")
                            .to_str()
                            .unwrap()
                            .split(".")
                            .last()
                            .unwrap()
                    }
                ));
            }
        }
        // If output file exists, delete it
        if args.output.to_str().expect("Failed to get output") != "-" {
            if args.output.exists() {
                fs::remove_file(&args.output).expect("Failed to delete existing output file.");
            }
        }
        // If there is set both fast and audio option, inform user that they are incompatible.
        if args.fast && args.audio {
            eprintln!("Audio option and fast option cannot be used together. Please use only one.");
            eprintln!(
                "It's strongly recommended to use the --audio option. Using only audio is faster in every case."
            );
            return;
        }

        if !args.quiet {
            eprintln!("Extracting video metadata");
        }

        // Get general video metadata
        let video_metadata: VideoMetadata = get_video_metadata(args.input.to_str().unwrap());

        let mut silent_frames: Vec<bool>;
        // Detect silent frames
        {
            if !args.quiet {
                eprintln!("Extracting audio");
            }

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

            if !args.quiet {
                eprintln!("Processing audio");
            }

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

            if !args.quiet {
                eprintln!(
                    "Found {} silent video frames out of total {} frames.",
                    silent_frames
                        .iter()
                        .filter(|f| **f)
                        .collect::<Vec<&bool>>()
                        .len(),
                    silent_frames.len()
                );
            }
        }

        // Compute speedup ranges
        let mut silent_segments_count: usize = 0;
        let mut audio_segments_speedup: Vec<SpeedupRange> = Vec::new();
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
            if silent_frames[0] {
                silent_segments_count += 1;
            }
            let mut current_speedup_loudness: bool = silent_frames[0];
            for i in 1..silent_frames.len() {
                if silent_frames[i] == current_speedup_loudness {
                    continue;
                } else {
                    current_speedup.frame_to = i;
                    audio_segments_speedup.push(current_speedup);
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
                    if silent_frames[i] {
                        silent_segments_count += 1;
                    }
                }
            }
            current_speedup.frame_to = silent_frames.len() - 1;
            audio_segments_speedup.push(current_speedup);
        }

        if !args.quiet {
            eprintln!(
                "Found {} silent video segments out of total {} segments.",
                silent_segments_count,
                audio_segments_speedup.len()
            );
        }

        // If user says so, estimate runtime, time saved,
        // print it and exit.
        if args.show_stats {
            let video_silent_frames = silent_frames
                .iter()
                .filter(|f| **f)
                .collect::<Vec<&bool>>()
                .len();
            let silent_percentage_of_video = video_silent_frames as f32 / silent_frames.len() as f32;
            println!(
                "{}% of video is silent.",
                silent_percentage_of_video * 100.0
            );
            println!(
                "It will take about {} seconds to process {} segments with flawless quality, or about {} seconds with watchable quality. Processing only audio will be almost instantaneous.",
                    video_metadata.duration_seconds as usize * 2,
                    audio_segments_speedup.len(),
                    audio_segments_speedup.len() / 3,
            );
            let time_total = video_metadata.duration_seconds;
            let raw_duration_in_silence = silent_percentage_of_video * time_total;
            let raw_duration_in_loudness = time_total - raw_duration_in_silence;
            let mut real_duration_in_silence = raw_duration_in_silence * (1.0 / args.speed_silent);
            let mut real_duration_in_loudness = raw_duration_in_loudness * (1.0 / args.speed_loud);
            if args.speed_silent >= 100.0 {
                real_duration_in_silence = 0.0;
            }
            if args.speed_loud >= 100.0 {
                real_duration_in_loudness = 0.0;
            }
            let real_duration = real_duration_in_silence + real_duration_in_loudness;
            println!(
                "Estimated time saved is {} minutes ({}%).",
                (time_total - real_duration) / 60.0,
                ((time_total - real_duration) / time_total as f32) * 100.0
            );
            return;
        }

        let video_segments_speedup: Vec<SpeedupRange>;
        // Figure out where to cut video
        {
            // Map speedup ranges to video frames
            let last_audio_frame = audio_segments_speedup.last().unwrap().frame_to;
            let last_video_frame = video_metadata.total_frames;
            let rate: f32 = last_video_frame as f32 / last_audio_frame as f32;
            video_segments_speedup = audio_segments_speedup
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

        // Tell ffmpeg to do it (slower, best resolution, doesn't use temp files)
        if !args.fast {
            let filter =
                generate_complex_speedup_filter(&video_segments_speedup, &video_metadata, args.audio);
            // Save filter to file
            // Create temporary directory where we will store temporary complex filter file.
            let tempdir_path = std::env::temp_dir().join(GUID::rand().to_string());
            fs::DirBuilder::new()
                .create(&tempdir_path)
                .expect("Failed to create tmp directory.");
            let filter_filename = tempdir_path.join("complex_filter.txt");
            fs::write(filter_filename.to_str().unwrap(), filter).unwrap();
                
            if !args.quiet {
                // Displaying "come back in N minutes" doesn't make sense with the --audio option, since it's really fast.
                if !args.audio {
                    eprintln!(
                        "Starting ffmpeg process. Come back in about {} minutes.",
                        (video_metadata.duration_seconds / 40.0) as usize
                    );
                    eprintln!("If you need result fast and don't care about resolution, use --fast flag. It's much faster and generally pretty watchable.");
                    eprintln!("If you don't need video, use the --audio flag. It will make the process almost instantaneous.")
                }
            }
            speedup_using_complex_filter(&args.input, &args.output, &filter_filename.to_str().unwrap(), args.audio);
            fs::remove_dir_all(&tempdir_path).expect("Failed to remove tmp directory.");
        } else
        // Do the splitting, speed-uping, etc manually (fastest, worst result)
        {
            // Create temporary directory where we will store everything.
            let tempdir_path = std::env::temp_dir().join(GUID::rand().to_string());
            fs::DirBuilder::new()
                .create(&tempdir_path)
                .expect("Failed to create tmp directory.");

            // Split and speedup videos, get these part names in order.
            let mut video_part_paths: Vec<Option<PathBuf>> = Vec::new();
            let mut current_part: f32 = 0.0;
            let parts_len = video_segments_speedup.len() as f32;
            for frame in video_segments_speedup {
                if !args.quiet {
                    eprintln!("{}%", (current_part / parts_len) * 100.0);
                }
                video_part_paths.push(speedup_video_part(
                    args.input.to_str().unwrap(),
                    &frame,
                    &video_metadata,
                    &tempdir_path,
                    args.fast,
                ));
                current_part += 1.0;
            }

            // Concatenate temp files
            concatenate_videos_to_file(
                video_part_paths
                    .iter()
                    .filter(|p| p.is_some())
                    .map(|p| (p.as_ref().unwrap().to_str().unwrap()))
                    .collect::<Vec<&str>>(),
                &tempdir_path,
                args.output,
            );

            fs::remove_dir_all(&tempdir_path).expect("Failed to remove tmp directory.");
        }
    }

    /// Scan video with ffprobe to determine video length, fps, and duration.
    /// This is used to sync audio and video and output estimate runtime.
    fn get_video_metadata(filename: &str) -> VideoMetadata {
        let duration_seconds_command = Command::new("ffprobe")
            .args(&[
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1",
                filename,
            ])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to get duration via ffprobe.");
        // Expected format: duration=2838.919000
        let duration_seconds_string = String::from_utf8(duration_seconds_command.stdout).unwrap();
        let duration_seconds_string = duration_seconds_string.split("=").last().unwrap().trim();
        let duration_seconds: f32 = duration_seconds_string.parse().expect(&format!(
            "Failed to parse video duration from {}",
            duration_seconds_string
        ));

        let fps_command = Command::new("ffprobe")
            .args(&[
                "-select_streams",
                "v",
                "-show_entries",
                "stream=r_frame_rate",
                "-of",
                "default=noprint_wrappers=1",
                filename,
            ])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to get fps via ffprobe.");
        // Expected format: r_frame_rate=30000/1001
        let fps_string = String::from_utf8(fps_command.stdout).unwrap();
        let fps_string = fps_string.split("=").last().unwrap();
        let fps_string_split = (
            fps_string.split("/").take(1).last().unwrap().trim(),
            fps_string.split("/").last().unwrap().trim(),
        );
        let fps_numbers_split: (f32, f32) = (
            fps_string_split
                .0
                .parse()
                .expect(&format!("Failed to parse video fps(1) from {}", fps_string)),
            fps_string_split
                .1
                .parse()
                .expect(&format!("Failed to parse video fps(2) from {}", fps_string)),
        );
        let fps = fps_numbers_split.0 / fps_numbers_split.1;

        let total_frames_command = Command::new("ffprobe")
            .args(&[
                "-select_streams",
                "v",
                "-show_entries",
                "stream=nb_frames",
                "-of",
                "default=noprint_wrappers=1",
                filename,
            ])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to get total number of frames via ffprobe.");
        let total_frames_string = String::from_utf8(total_frames_command.stdout).unwrap();
        let total_frames_string = total_frames_string.split("=").last().unwrap().trim();
        let total_frames_result: Result<usize, std::num::ParseIntError> =
            total_frames_string.parse::<usize>();
        let total_frames: usize =
            total_frames_result.unwrap_or(duration_seconds as usize * fps as usize);

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
    /// If speed is higher or equal to 100, return `None`.
    fn speedup_video_part(
        input_path: &str,
        range: &SpeedupRange,
        metadata: &VideoMetadata,
        tempdir_path: &std::path::Path,
        force_mpeg: bool,
    ) -> Option<PathBuf> {
        if range.speedup_rate < 0.5 {
            panic!("Fatal error: speed rate is lower than 0.5.");
        }
        if range.speedup_rate >= 100.0 {
            return None;
        }

        // Sometimes things get wrong and we are said to cut video with 0 frames length
        // Don't do anything in that case.
        if range.frame_to - range.frame_from == 0 {
            return None;
        }

        let extension = if force_mpeg {
            "mpeg"
        } else {
            input_path.split(".").last().unwrap().trim()
        };

        let cut_video_filename = format!("{}.{}", GUID::rand().to_string(), extension);
        let speedup_video_filename = format!("{}.{}", GUID::rand().to_string(), extension);
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
                extension,
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
                extension,
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

    /// Create file that will contain all video names in given directory.
    /// Afterwards, concatenate all those videos using ffmpeg to output path.
    fn concatenate_videos_to_file(filenames: Vec<&str>, tempdir_path: &PathBuf, output_path: PathBuf) {
        // Create "files" file, which will contain list of filenames. We
        // will then pass this file to ffmpeg. We cannot do this normally,
        // since there is a limit on number of arguments ffmpeg can process
        // the old way.
        let extension = filenames.first().unwrap().split(".").last().unwrap();

        let filenames_register_path = tempdir_path.join("files.txt");
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&filenames_register_path)
            .expect("Failed to open file register.")
            .write_all(
                filenames
                    .iter()
                    .map(|x| format!("file '{}'", x))
                    .collect::<Vec<String>>()
                    .join("\n")
                    .as_bytes(),
            )
            .expect("Failed to write to file register.");

        Command::new("ffmpeg")
            .args(&[
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                tempdir_path.join("files.txt").to_str().unwrap(),
                "-f",
                extension,
                output_path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to run video concatenate process")
            .wait()
            .expect("Failed to concatenate video files.");
    }

    fn speedup_using_complex_filter(
        input: &PathBuf,
        output: &PathBuf,
        complex_filter_filename: &str,
        audio_only: bool,
    ) {
        let args: Vec<&str>;
        if audio_only {
            args = vec![
                "-i",
                input.to_str().unwrap(),
                "-vn",
                "-threads",
                "8",
                "-filter_complex_script",
                complex_filter_filename,
                "-f",
                input.to_str().unwrap().split(".").last().unwrap(),
                "-movflags",
                "frag_keyframe+empty_moov",
                output.to_str().unwrap(),
            ];
        } else {
            args = vec![
                "-i",
                input.to_str().unwrap(),
                "-preset",
                "faster",
                "-crf",
                "27",
                "-threads",
                "8",
                "-filter_complex_script",
                complex_filter_filename,
                "-f",
                input.to_str().unwrap().split(".").last().unwrap(),
                "-movflags",
                "frag_keyframe+empty_moov",
                output.to_str().unwrap(),
            ];
        }

        Command::new("ffmpeg")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn speedup via complex filter.")
            .wait()
            .expect("Failed to run speedup via complex filter.");
    }

    /// Generate ffmpeg complex filter that will speed up the video.
    ///
    /// For example, to speed up video that is one second long, in such way
    /// that first segment `(0.00 - 0.25)` will have double speed,
    /// second segment `(0.25 - 0.75)` will have standart speed and
    /// thrid segment `(0.75 - 1.00)` will have double speed, the complex
    /// filter will look like this:
    ///
    /// ```
    /// [0:v]trim=0:0.25,setpts=0.5*(PTS_STARTPTS)[v1];
    /// [0:a]atrim=0:0.25,asetpts=PTS-STARTPTS,atempo=2[a1];
    /// [0:v]trim=0.25:0.75,setpts=1*(PTS_STARTPTS)[v2];
    /// [0:a]atrim=0.25:0.75,asetpts=PTS-STARTPTS,atempo=1[a3];
    /// [0:v]trim=0.75:1,setpts=0.5*(PTS_STARTPTS)[v3];
    /// [0:a]atrim=0.75:1,asetpts=PTS-STARTPTS,atempo=2[a3];
    /// [v1][a1][v2][a2][v3][a3]concat=n=3:v=1:a=1
    /// ```
    fn generate_complex_speedup_filter(
        ranges: &Vec<SpeedupRange>,
        metadata: &VideoMetadata,
        audio_only: bool,
    ) -> String {
        let mut complex_filter = String::new();
        let mut idx: usize = 1;
        for range in ranges {
            if range.frame_to - range.frame_from == 0 {
                continue;
            }
            let seconds_from: f32 = range.frame_from as f32 / metadata.fps;
            let seconds_to: f32 = range.frame_to as f32 / metadata.fps;
            let inverted_speedup = 1.0 / range.speedup_rate;
            if !audio_only {
                complex_filter.push_str(&format!(
                    "[0:v]trim={}:{},setpts={}*(PTS-STARTPTS)[v{}];",
                    seconds_from, seconds_to, inverted_speedup, idx
                ));
            }
            complex_filter.push_str(&format!(
                "[0:a]atrim={}:{},asetpts=PTS-STARTPTS,atempo={}[a{}];",
                seconds_from, seconds_to, range.speedup_rate, idx
            ));
            idx += 1;
        }
        for i in 1..idx {
            if audio_only {
                complex_filter.push_str(&format!("[a{}]", i));
            } else {
                complex_filter.push_str(&format!("[v{}][a{}]", i, i));
            }
        }
        if audio_only {
            complex_filter.push_str(&format!("concat=n={}:a=1:v=0", idx - 1));
        } else {
            complex_filter.push_str(&format!("concat=n={}:v=1:a=1", idx - 1));
        }

        complex_filter
    }

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
}
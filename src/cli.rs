use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "Video Summarizer",
    about = "Take a video, and change it's speed, depending on silent and loud parts.\n\nTl;dr: >> video-summarizer video.mp4 -l 2 -s 4 << takes a video, speeds up loud parts 2x, silent parts 4x, and plays it in VLC.\nUse deprecated -o option to save video into file (slow!).",
    rename_all = "kebab-case"
)]
pub struct Cli {
    /// Source video
    ///
    /// Path to source video. Video must be
    /// parsable by FFMPEG.
    #[structopt(parse(from_os_str))]
    pub input: std::path::PathBuf,
    /// Output file (deprecated)
    ///
    /// Please note that this option will use fallback FFMPEG solution,
    /// which is really slow.
    /// 
    /// Don't use this option to allow new VLC method.
    #[structopt(parse(from_os_str), short = "o", default_value = "")]
    pub output: std::path::PathBuf,
    /// Video speed when loud sound is detected.
    ///
    /// This has to be at least 0.5.
    /// If this is larger than or equal to 100, loud parts of
    /// the video will be dropped completely.
    #[structopt(long = "speed-loud", short = "l", default_value = "1.5")]
    pub speed_loud: f32,
    /// Video speed when no loud sound was detected.
    ///
    /// This has to be at least 0.5.
    /// If this is larger than or equal to 100, silent parts
    /// of the video will be dropped completely.
    #[structopt(long = "speed-silent", short = "s", default_value = "5")]
    pub speed_silent: f32,
    /// Threshold of silence. When sound gets under this threshold,
    /// current frame will be considered as silent.
    ///
    /// If it sounds as if the speech is cut out right at start/end,
    /// consider editing "frame-margin" option first.
    #[structopt(long = "silence-threshold", default_value = "0.02")]
    pub silence_threshold: f32,
    /// Number of frames before/after loud frames to be considered
    /// loud as well, even if they actually aren't.
    ///
    /// Use this settings if beginning/end of sentences
    /// get cut out/sped up as they are considered silent.
    #[structopt(long = "frame-margin", default_value = "2")]
    pub frame_margin: usize,
    /// Do not print progress information.
    #[structopt(long = "quiet", short = "q")]
    pub quiet: bool,
    /// Do not do anything, just print information about the video. (deprecated)
    /// 
    /// This includes estimated run time and time saved on the video.
    /// This option is deprecated. It will be ignored unless --output (-o) option is specified.
    #[structopt(long = "stats")]
    pub show_stats: bool,
    /// Encode resulting video in MPEG. This will probably make resolution
    /// worse, but will speed up the whole process a LOT. (deprecated)
    ///
    /// This option is obsolete. It doesn't support piping out
    /// (immediatelly, you'll have to wait for the processing to end first)
    /// and it doesn't support the --audio option.
    /// This option is deprecated. It will be ignored unless --output (-o) option is specified.
    #[structopt(long = "fast")]
    pub fast: bool,
    /// Keep only audio, and drop all video frames. This will
    /// make processing almost instantaneous. (deprecated)
    /// 
    /// This option is deprecated. It will be ignored unless --output (-o) option is specified.
    #[structopt(long = "audio")]
    pub audio: bool,
}
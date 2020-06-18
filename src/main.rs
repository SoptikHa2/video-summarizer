mod mpv_controller;
use structopt::StructOpt;
use std::sync::mpsc::{self, Sender, Receiver};

#[derive(StructOpt)]
#[structopt(name="Video Summarizer", about="Take video and play it in MPV, speed up based on importance of various parts of the video. Important parts are plalyed at different rate than silent and non-important ones.")]
struct Opt {
    /// Input file to play. This
    /// has to be on disk.
    /// Piping video in is in TODO state.
    video_inputfile: String,
    /// How much to speed up the video that consists of loud
    /// (and therefore important) parts.
    #[structopt(short="l", default_value="2.0")]
    speed_loud: f64,
    /// How much to speed up the video that consists of silent
    /// (and therefore not important) parts.
    #[structopt(short="s", default_value="3.0")]
    speed_silent: f64,
}

fn main() {
    let opt = Opt::from_args();

    let (tx, rx) = mpsc::channel::<f64>();
    let mut controller = mpv_controller::MpvController::new(&opt.video_inputfile, rx).unwrap();
    controller.start_playing();
}

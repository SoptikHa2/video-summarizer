mod cli;
use cli::Cli;
mod fallback_ffmpeg;

use structopt::StructOpt;

fn main() {
    let mut args: Cli = Cli::from_args();

    if args.output.to_str().expect("Failed to get output filename").len() != 0 {
        // User specified output filename.
        // Fallback to ffmpeg.
        eprintln!("Warning: using deprecated ffmpeg fallback. For the supported version, install vlc and don't use -o option.");
        fallback_ffmpeg::fallback_ffmpeg::solve(args);
        return;
    }

    unimplemented!("New solution TODO");
}
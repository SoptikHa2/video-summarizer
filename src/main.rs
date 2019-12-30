mod cli;
use cli::Cli;
mod fallback_ffmpeg;

use structopt::StructOpt;

fn main() {
    let mut args: Cli = Cli::from_args();

    if args
        .output
        .to_str()
        .expect("Failed to get output filename")
        .len()
        != 0
    {
        // User specified output filename.
        // Fallback to ffmpeg.
        if !args.quiet {
            eprintln!("Warning: using deprecated ffmpeg fallback. For the supported version, install vlc and don't use -o option.");
        }
        fallback_ffmpeg::fallback_ffmpeg::solve(args);
        return;
    }

    // Warn user of deprecated options
    if !args.quiet {
        if args.audio {
            eprintln!("Warning: using deprecated --audio flag that has no effect in this context.");
        }
        if args.fast {
            eprintln!("Warning: using deprecated --fast flag that has no effect in this context.");
        }
        if args.show_stats {
            eprintln!("Warning: using deprecated --stats flag that has no effect in this context.");
        }
    }

    unimplemented!("New solution TODO");
}

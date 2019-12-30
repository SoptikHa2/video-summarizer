mod cli;
use cli::Cli;
mod fallback_ffmpeg;

use structopt::StructOpt;

// How does this work?
//
// We have two different implementations right now.
//
// The first one, deprecated one (as seen in fallback_ffmpeg.rs) works like this:
// We analyze the video audio. When the audio volume is not moving too much, it's either
// silence or background noise. We declare these parts of video as silent.
// If the opposite is true, the parts are deemed to be loud.
// We map video and audio frames together (hint: there is different number of audio and video frames)
// and compute when should we make video faster/slower and by how much.
// Now, we either create ffmpeg complex filter (https://trac.ffmpeg.org/wiki/How%20to%20speed%20up%20/%20slow%20down%20a%20video)
// that slows down/speeds up video+audio at certain timestamps. We render it and we're done.
// The disadvantage is that ffmpeg uses just one thread for the operation, so it's rather slow. And
// due to my terrible code (I was learning rust), one can't really pipe it into video viewer.
// Well, can, but the processing speed is still slower than playback speed.
// If we want to be faster (but the quality will be awful), we can convert video to mpeg, split it,
// speed up each part individually and concatenate them withotu ffmpeg. This is faster, but the quality
// is really low and uses lot of temp files. I don't really like this solution, but it's an (`--fast`) option.
//
// But we have second solution: the VLC one.
// VLC is mighty video player. And it offers one interesting flag: -Irc.
// This allows one to send commands/receive info via stdin/stdout while still
// playing the video as normal. So we can just analyze the sound and tell VLC
// to play the video. Than during video playback, we check for video position (`get_time`)
// (as user might skip some time or return playback back), whether user paused the video (`is_playing`)
// and adjust video speed (`rate`).
// And if we rewrite audio analysis into multiple threads, this might be actually pretty quick and painless.
// The downside is that we no longer can save the video (except for vlc record feature), so we need to use
// the first option as a fallback.

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

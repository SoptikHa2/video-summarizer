This application lets user speed up video at varying rates, based on current loudness. For example, I can speed up loud parts of a lecture 1.5x, and the silent parts 5x. Application does this by taking audio and search where the audio has [suspiciously constant level](https://imgur.com/Y2rzUkK) for big amount of time. Afterwards, I just split it, speed it up, and concatenate it via ffmpeg. This was done primarily to learn more about Rust, but the result are actually much better than I thought. I think this would actually be viable to use in case of internet lectures.

New version of this package actually uses VLC instead of FFMPEG to speed up the vidoe. This doesn't offer saving the video into file (which is done via ffmpeg), but is much, much faster.

See the Results section for analysis I did few versions ago.

# Usage

Play a video in vlc. Seed up loud parts 1.5x and silent parts 5x.
```sh
video-summarizer -l 1.5 -s 5 "video.mp4"
```

Play a video in vlc. Don't speed up loud parts and cut out silent parts. (If speed is >= 100, the loud/silent parts are cut of the video.)
```sh
video-summarizer -s 100 "video.wmv"
```

Play a video in vlc. Slow down silent parts 2x, for whatever reason.
```sh
video-summarizer -s 0.5 "video.webm"
```

You can specify video output as a file, but in that case, processing takes quite a long time. We fallback to old method which is really slow. We don't recommend this option, play it in VLC whenever you can.

Convert multiple lecture files. Speed up loud parts 1.5x and silent parts 5x.

```sh
for lecture-video in *.mp4; do
	video-summarizer -l 1.5 -s 5 "$lecture_video" -o "NEW-$lecture_video"
done
```

I don't care about quality, I need it NOW.

```sh
video-summarizer -l 2.5 -s 7 --fast video.mp4 -o new-video.mp4
```

Cut silent parts out of a video.

```sh
video-summarizer -s 100 video.mp4 -o video-cut.mp4
```

Get only audio from a video. This option is really fast.

```sh
video-summarizer -l 2 -s 4 video.mp4 --audio -o audio-only.mp3
```

# Install

Make sure you have required dependencies and either download binary from releases, or build it yourself. I suggest you to download binary if you want to just try it, but the best option is building directly from Rust repository. Everything, including updates, is taken care of.

Install [ffmpeg](https://ffmpeg.org/download.html) [\[apt\]](https://packages.ubuntu.com/search?keywords=ffmpeg&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86\_64/ffmpeg/) in order to use this program.

We strongly recommend you to install [vlc](https://www.videolan.org/vlc/)  [\[apt\]](https://packages.ubuntu.com/search?suite=all&section=all&arch=any&keywords=vlc&searchon=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86_64/vlc/)

If you want to build it yourself, you'll need to install [rust](https://www.rust-lang.org/) [\[apt\]](https://packages.ubuntu.com/search?keywords=rust&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86\_64/rust/) and [git](https://git-scm.com/downloads) [\[apt\]](https://packages.ubuntu.com/search?keywords=git&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86_64/git/) as well.

## Build from Rust repository

```
cargo install video-summarizer
```

## Build from source


```
# Clone this repository
git clone https://github.com/SoptikHa2/video-summarizer.git
```

```
# Compile debug build to verify everything works
cd video-summarizer
cargo build
target/debug/video-summarizer -h
```

```
# Install release build
cargo install
```

```
# Run
# this will speedup loud parts 1.5x and completely drop silent parts (as speedup factor is >= 100)
video-summarizer -l 1.5 -s 100 path/to/video
```

# Dependencies

## Build dependencies

- [Rust](https://www.rust-lang.org/) [\[apt\]](https://packages.ubuntu.com/search?keywords=rust&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86\_64/rust/)

## Runtime dependencies
- [ffmpeg](https://wiki.archlinux.org/index.php/FFmpeg/) (tested on 4.2) [\[apt\]](https://packages.ubuntu.com/search?keywords=ffmpeg&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86\_64/ffmpeg/)

- [vlc](https://www.videolan.org/vlc/) (tested on 3.0.8) [\[apt\]](https://packages.ubuntu.com/search?suite=all&section=all&arch=any&keywords=vlc&searchon=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86_64/vlc/) (optional, but strongly recommended)

# Results

Everything was tested with ffmpeg 4.2, and video summarizer 1.1.1. Settings: `-l 1.5 -s 100`.

| Name | Duration (s) | Silent time (%) | Saved time (s) |
|---|---|---|---|
|  [DEFCON 17: That Awesome Time I Was Sued For Two Billion Dollars](https://www.youtube.com/watch?v=KSWqx8goqSY) |  1887 | 15.32% | 822 (43.55%) |
|  [1. Introduction and Scope](https://www.youtube.com/watch?v=TjZBTDzGeGg&t=124s) (MIT AI course) | 2838 |  40.12% | 1706 (60.08%) |
| [Black Mirror: White Christmas ](https://www.imdb.com/title/tt3973198/) | 4326 | 11.93% | 1786 (41.29%) |
| [Puella Magi Madoka Magica Ep 10](https://www.imdb.com/title/tt1773185/) | 1449 | 7.22% | 553 (38.15%) |

# Known issues
- Piping fails for some reason most of the time. This is in WONTFIX state. If someone fixes it, I'll accept the patch, but I won't fix it myself. The priority is smooth playback with VLC right now.

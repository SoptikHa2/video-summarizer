This application lets user speed up video at varying rates, based on current loudness. For example, I can speed up loud parts of a lecture 1.5x, and the silent parts 5x. Application does this by taking audio and search where the audio has [suspiciously constant level](https://imgur.com/Y2rzUkK) for big amount of time. Afterwards, I just split it, speed it up, and concatenate it via ffmpeg. This was done primarily to learn more about Rust, but the result are actually much better than I thought. I think this would actually be viable to use in case of internet lectures.

# Install

Make sure you have required dependencies and either download binary from releases, or build it from source.

## Build from source

Install [rust](https://www.rust-lang.org/) [\[apt\]](https://packages.ubuntu.com/search?keywords=rust&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86\_64/rust/)

Install [git](https://git-scm.com/downloads) [\[apt\]](https://packages.ubuntu.com/search?keywords=git&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86_64/git/)

Install [ffmpeg](https://ffmpeg.org/download.html) [\[apt\]](https://packages.ubuntu.com/search?keywords=ffmpeg&searchon=all&suite=all&section=all) [\[pacman\]](https://www.archlinux.org/packages/extra/x86\_64/ffmpeg/)

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

# Results

Everything was tested with ffmpeg 4.2, and video summarizer 1.1.0. Settings: `-l 1.5 -s 100`.

| Name | Duration (s) | Silent time (%) | Saved time (s) |
|---|---|---|---|
|  [DEFCON 17: That Awesome Time I Was Sued For Two Billion Dollars](https://www.youtube.com/watch?v=KSWqx8goqSY) |  1887 | 15.32% | 822 (43.55%) |
|  [1. Introduction and Scope](https://www.youtube.com/watch?v=TjZBTDzGeGg&t=124s) (MIT AI course) | 2838 |  40.12% | 1706 (60.08%) |
| [Black Mirror: White Christmas ](https://www.imdb.com/title/tt3973198/) | N/A | N/A | N/A |
| [Puella Magi Madoka Magica Ep 10](https://www.imdb.com/title/tt1773185/) | 1449 | 7.22% | 553 (38.15%) |

*(I'll update Black Mirror test case later)*

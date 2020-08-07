# Video summarizer

This can be used to play videos at varying speeds based on how much is the video interesting. So far there is no neural net in the backend, it's based on loundness. So no more long seconds watching teacher writing on whiteboard.

INSERT FANCY GIF HERE

For example, one can watch a long lecture, playing the parts where someone is talking at `1.5` rate, and skip the silent parts with `5x` rate.

## Usage

There are multiple sections here in the repo.

- [rust-desktop-cli](rust-desktop-cli) (unmantained obsolete cli. It works, but uses ffmpeg and is a bit slow)
- [extension-backend](extension-backend) (shell & rust backend that analyzes video sound and caches it, but doesn't do much else)
- [extension-firefox](extension-firefox) (firefox extension that speeds up videos based on backend response)

## Results

Everything was tested with ffmpeg 4.2, and video summarizer 1.1.1 (the rust cli). Settings: `-l 1.5 -s 100`.

| Name | Duration (s) | Silent time (%) | Saved time (s) |
|---|---|---|---|
|  [DEFCON 17: That Awesome Time I Was Sued For Two Billion Dollars](https://www.youtube.com/watch?v=KSWqx8goqSY) |  1887 | 15.32% | 822 (43.55%) |
|  [1. Introduction and Scope](https://www.youtube.com/watch?v=TjZBTDzGeGg&t=124s) (MIT AI course) | 2838 |  40.12% | 1706 (60.08%) |
| [Black Mirror: White Christmas ](https://www.imdb.com/title/tt3973198/) | 4326 | 11.93% | 1786 (41.29%) |
| [Puella Magi Madoka Magica Ep 10](https://www.imdb.com/title/tt1773185/) | 1449 | 7.22% | 553 (38.15%) |

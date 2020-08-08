# Video summarizer <img src="extension-firefox/icon.svg" align="right" />

Changes playback speed for videos depending on loudness of the video. Speed up long lectures at different rates, depending on whether the teacher is saying something or is just silently writing something on whiteboard.

<img src="ff-vidsum-ui.png" align="right" />

This extension analyses video (typically youtube one, but shall work with any HTML video) and, depending on current loundness of the video, speeds it up at different rates. One can for example speed up teacher talking at 1.5x rate, and speed up teacher writing something at whiteboard at 4x rate.

This saves quite a bit of time, especially during watching long lectures.

[Get it for Firefox](https://addons.mozilla.org/en-US/firefox/addon/video-summarizer/) (Important: Videos that are controlled by this extension are stored on my server. So most videos are not indexed and thus not managed by this extension yet! To index a video, navigate to it and click on the addon icon located next to URL bar. After few seconds, the video should be ready to view through the extension.)

Google Chrome and chromium-based browsers are not currently supported. Mostly because it costs money to buy developer account. [Support me](https://paypal.me/stastnysoptik) if you like my work. If I receive 25USD or whatever google development account costs, I'll make a chrome extension.

## How does it work

There are two parts to this extension: frontend and backend. Backend indexes videos: downloads them via youtube-dl, extracts sound with ffmpeg and analyzes it with shell and awk (yes, the server is written entirely with gnu coreutils). Generally we seek parts of videos that are greatly below or above average sound of the video. This way, we can determine which parts of video are loud and which are silent.

When frontend navigates to a page, it checks if there is a HTML5 video. If so, it hashes current URL (after removing uninteresting query parameters and such) and sends it to server, which determines if it has current video cached. Frontend may receive response, which tells it which parts of video are loud or silent. In the opposite case, a button in extension popup panel (the thing that appears when one clicks the extension button next to url bar) appears that allows user to submit video to server for indexation. This generally takes about ten seconds for short videos (sub-10 minutes).

## Usage

There are multiple sections here in the repo.

- [rust-desktop-cli](rust-desktop-cli) (unmantained obsolete cli. It works, but uses ffmpeg and is a bit slow)
- [extension-backend](extension-backend) (shell & gnu coreutils-powered backend that analyzes video sound, caches it, and serves via http server)
- [extension-firefox](extension-firefox) (firefox extension that speeds up videos based on backend response) [Get it for Firefox](https://addons.mozilla.org/en-US/firefox/addon/video-summarizer/)

## Results

Everything was tested with ffmpeg 4.2, and video summarizer 1.1.1 (the rust cli). Settings: `-l 1.5 -s 100`.

| Name | Duration (s) | Silent time (%) | Saved time (s) |
|---|---|---|---|
|  [DEFCON 17: That Awesome Time I Was Sued For Two Billion Dollars](https://www.youtube.com/watch?v=KSWqx8goqSY) |  1887 | 15.32% | 822 (43.55%) |
|  [1. Introduction and Scope](https://www.youtube.com/watch?v=TjZBTDzGeGg) (MIT AI course) | 2838 |  40.12% | 1706 (60.08%) |
| [Black Mirror: White Christmas ](https://www.imdb.com/title/tt3973198/) | 4326 | 11.93% | 1786 (41.29%) |
| [Puella Magi Madoka Magica Ep 10](https://www.imdb.com/title/tt1773185/) | 1449 | 7.22% | 553 (38.15%) |

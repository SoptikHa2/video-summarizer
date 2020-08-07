# Video summarizer extension backend

This remembers cache of videos and their speedup rates. So far, user-triggered video speedup rate generating is not implemented.

Video caches are stored in files, named by SHA1 of URL of the video (for example youtube video).

The cache is generated this way: Audio is downloaded via youtube-dl, audio levels are extracted via ffmpeg and stored into `$SHA1SUM.audiolevels`.
It is then passed to rust script, which reads it and assigns either `0` (silent) or `1` (loud) to each N-th frame of the video and saves it as `$SHA1SUM`.
When this is ready, it can be read by the shell server and served to extension which speeds up or slows down video based on it.

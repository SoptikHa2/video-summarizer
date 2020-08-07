# Video summarizer extension backend

This remembers video speedup rates and serves as a cache. So far, user-triggered video speedup rate generating is not implemented.

Video caches are stored in files, named by SHA1 of URL of the video (for example youtube video).

The cache is generated this way: Audio is downloaded via youtube-dl, audio levels are extracted via ffmpeg and stored into `$SHA1SUM.audiolevels`.
It is then passed to rust script, which reads it and assigns either `0` (silent) or `1` (loud) to each N-th frame of the video and saves it as `$SHA1SUM`.
When this is ready, it can be read by the shell server and served to extension which speeds up or slows down video based on it.

---

Bash server note: these notes from `protab.cz` were used for writing the server.
```
a='abcdefghijklmnop'
echo ${a:0:3}  # Od nulteho tri
echo ${a:4:7}  # Od ctvrteho sedm
echo ${a:4:4}  # Od ctvrteho ctyri
echo ${a:4:-1} # Od ctvrteho do predposledniho
echo ${a: -3} # Posledni tri znaky
echo ${a:-3} # Kdyz a je unset, vrat '3'
echo ${a: -3:1} # Posledni 3 a od toho jeden znak


# Chci sezrat ze zacatku retezce znak
echo ${a#*f} # Seberu ze zacatku znaky, funguje glob
echo ${a%.*} # Sebere z konce (odstrani priponu)
echo ${a%%.*} # Nejdelsi match
echo ${a##*f} # Nejdelsi match

echo ${a/a/b} # Nahradi a na b
echo ${a//a/b} # Nahradi vsechny a na b
echo ${a//a} # Sebere acka
echo ${a/a} # Sebere acko
echo ${a^^} # Vsechno kapitalizace
echo ${a^} # Prvni na velky
echo ${a,} # Prvni na maly
```

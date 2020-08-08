#!/bin/bash
# Process url at $1
set -euo pipefail

urlhash=$(sha1sum <<<"$1" | cut -d' ' -f1)

if [ -e "videocache/$urlhash" ]; then
    echo "There already exists this url with hash $urlhash. Delete it to recompute." >&2
    exit 1
fi

sound_levels="$(youtube-dl "$1" -f worstaudio -o - | ffmpeg -i - -af astats=metadata=1:reset=1,ametadata=print:key=lavfi.astats.Overall.RMS_level:file=- -f null -)"

echo "$sound_levels" | grep -Eo -- '-?(inf)?[0-9]*(\.[0-9]+)?$' | sed 'N;s/\n/ /' | grep -v inf | awk '$2 > -50 { print $0 }' > "videocache/$urlhash.audiolevels"

# Process the audio levels

avg_min_max="$(awk 'BEGIN{min="";max=""}{s+=$2;if(min==""||$2<min){min=$2}if(max==""||$2>max){max=$2}} END{print s/NR " " min " " max}' "videocache/$urlhash.audiolevels")"
avg=$(echo "$avg_min_max" | cut -d' ' -f1)
min=$(echo "$avg_min_max" | cut -d' ' -f2)
max=$(echo "$avg_min_max" | cut -d' ' -f3)
# Arbitrary heuristic-based number. Anything above avg-threshold will be considered loud.
threshold="$(echo "($max-($min))*0.2" | bc)"
loud_cutoff="$(echo "$avg-($threshold)" | bc)"

awk -v "threshold=$loud_cutoff" '
$2 > threshold {
    print $1 " 1"
}
$2 <= threshold {
    print $1 " 0"
}
' "videocache/$urlhash.audiolevels" > "videocache/$urlhash"

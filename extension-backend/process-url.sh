#!/bin/bash
# Process url at $1

# Force rewrite if there already exists a record. This can be specified as $2="--recompute". This also assumes user
# will specify hash directly instead of URL and the audiolevels file already exists
force_recompute=0
if [[ -n "$2" ]] && [[ "$2" == "--recompute" ]]; then
    force_recompute=1
fi

set -euo pipefail

urlhash="$1"
if [[ $force_recompute -eq 0 ]]; then
    # NOTE: Here we need to do the printf and pipe, or else the hash is different - probably because of some newline at end for some reason?
    urlhash=$(printf "$1" | sha1sum | cut -d' ' -f1)

    if [ -e "videocache/$urlhash" ]; then
        echo "There already exists this url with hash $urlhash. Delete it to recompute." >&2
        exit 1
    fi

    # If we didn't yet download the video, do so now
    if [ ! -e "videocache/$urlhash.audiolevels" ]; then
        sound_levels="$(youtube-dl "$1" -f worstaudio -o - | ffmpeg -i - -af astats=metadata=1:reset=1,ametadata=print:key=lavfi.astats.Overall.RMS_level:file=- -f null -)"

        echo "$sound_levels" | grep -Eo -- '-?(inf)?[0-9]*(\.[0-9]+)?$' | sed 'N;s/\n/ /' | grep -v inf | awk '$2 > -50 { print $0 }' > "videocache/$urlhash.audiolevels"
    fi
fi

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

# Smooth playback
# Just extend each loud pattern by X frames to left and right
extend_loudness=2
function smooth_forward {
    gawk -i inplace -v "n=$extend_loudness" '
    BEGIN { beg=-100000 }
    NR <= (beg+n) {
        print $1 " 1"
    }
    NR > (beg+n) {
        print $1 " " $2
    }
    $2 == 1 {
        beg=NR
    }
    ' "videocache/$urlhash"
}
smooth_forward
tmptac=$(mktemp)
tac "videocache/$urlhash" > "$tmptac"
cat "$tmptac" > "videocache/$urlhash"
smooth_forward
tac "videocache/$urlhash" > "$tmptac"
cat "$tmptac" > "videocache/$urlhash"
rm -f "$tmptac"

# Only store volume switches, and not every frame
gawk -i inplace '
BEGIN { lastVolume=-1 }
$2 != lastVolume {
    print $0;
    lastVolume = $2;
}
' "videocache/$urlhash"


# We want to keep it for now. As heuristic will probably change often
# while this is in development and I don't want to redownload videos
# in order to recompute everything
# rm "videocache/$urlhash.audiolevels"

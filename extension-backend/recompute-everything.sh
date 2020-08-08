#!/bin/bash
for file in videocache/*; do
    if [[ ! "$file" == "*.audiolevels" ]]; then
        f=$(basename "$file")
        echo "spawning ./process-url.sh $f --recompute"
        ./process-url.sh "$f" --recompute
    fi
done


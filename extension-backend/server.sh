#!/bin/bash
# Shell server adapted from ynaas: https://github.com/izabera/ynaas
set -euo pipefail

host=videosummarizer.soptik.tech
IFS=$' \t\n\r'

read -r request destination_and_protocol
destination=${destination_and_protocol% *}

while read -r header value; do
    [[ $header = Host: ]] && host=$value
    [[ $header = Content-Length: ]] && length=$value
    [[ $header ]] || break
done

case $request in
    GET)
        # Disallow bad destination names
        # Throw the request away if it contains dot, percent sign, dollar or a backtick
        # I don't want to validate percent-encoding in bash
        if [[ $destination == *.* ]] || [[ $destination == *%* ]] || [[ $destination == *\`* ]] || [[ $destination == *\$* ]]; then
            printf 'HTTP/1.1 400\r\n\r\n<img src="https://http.cat/400">Please do not use dots, dollar signs, backticks or percent encoding.'
            exit
        fi
        # Throw away all the slashes. Save just the first string inbetween slashes into $destination
        destination=${destination#/}
        destination=${destination%%/*}
        # Check if the video exists
        if [[ -n "$destination" ]] && [[ -e "videocache/$destination" ]]; then
            val=$(cat "videocache/$destination")
            printf 'HTTP/1.1 200 OK\r\n\r\n%s' "$val"
        else
            printf 'HTTP/1.1 404 Not Found\r\n\r\n%s' "<img src='https://http.cat/404'>"
        fi
        ;;
    POST)
        printf 'HTTP/1.1 403 Forbidden\r\n\r\nThis can only be run locally.\n'
        ;;

    *)
        printf 'HTTP/1.1 405 Method Not Allowed\r\nAllow: GET, POST\r\n\r\n<img src="https://http.cat/405" />'
        exit
        ;;
esac


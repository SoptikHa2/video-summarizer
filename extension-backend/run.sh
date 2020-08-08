#!/bin/sh

(
cd "$(dirname "$0")" || exit 1
mkdir -p videocache
ncat -e ./server.sh -kl 9920
)

#!/bin/sh

(
cd "$(dirname "$0")" || exit 1
ncat -e ./server.sh -kl 9920
)

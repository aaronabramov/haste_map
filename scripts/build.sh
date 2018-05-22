#!/bin/sh

BASEDIR=$(realpath $(dirname "$0")"/..")
source "$BASEDIR""/scripts/env.sh"

if [ "$1" == "release" ]; then 
    cargo build --release
    printf "[  ""$GREEN""OK""$NC""  ] Build ""$BLUE""optimized""$NC""\n"
    ./scripts/link_node_extension.sh release
else
    cargo build 
    printf "[  ""$GREEN""OK""$NC""  ] Build ""$BLUE""unoptimized""$NC""\n"
    ./scripts/link_node_extension.sh
fi

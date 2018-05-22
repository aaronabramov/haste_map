#!/bin/sh

BASEDIR=$(realpath $(dirname "$0")"/..")

source "$BASEDIR""/scripts/env.sh"

TARGET_DIR='debug'
BUILD_DIR="$BASEDIR""/build"
NODE_EXT=$(realpath -m "$BUILD_DIR""/libnodejs_extension.node")

if [ "$1" == "release" ]; then TARGET_DIR='release'; fi

DLYB=$(realpath "$BASEDIR""/target/""$TARGET_DIR""/libnodejs_extension.dylib")

rm -f $NODE_EXT
mkdir -p $BUILD_DIR

cp $DLYB $NODE_EXT  && printf "[  ""$GREEN""OK""$NC""  ] Create ""$BLUE""$NODE_EXT""$NC""\n"
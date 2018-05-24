#!/bin/sh

PROJECT_ROOT=$1

cargo build --release
rm -rf cache/
rm haste_map_js.txt
rm haste_map_rs.txt

node jest_haste_map/index.js $PROJECT_ROOT

./target/release/haste_map $PROJECT_ROOT
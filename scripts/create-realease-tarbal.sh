#!/bin/sh

cp target/release/peertube-viewer-rs peertube-viewer-rs
mkdir linux-build
tar -czf linux-build/peertube-viewer-rs-"$(git describe --tags)".tar.gz completions/* peertube-viewer-rs peertube-viewer-rs.1 LICENSE

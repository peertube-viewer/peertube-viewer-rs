#!/bin/sh

cp target/release/peertube-viewer-rs peertube-viewer-rs
mkdir linux-build
tar --transform 'flags=r;s|COPYING.md|LICENSE|' -czf linux-build/peertube-viewer-rs-"$(git describe --tags)".tar.gz completions/* peertube-viewer-rs peertube-viewer-rs.1 COPYING.md

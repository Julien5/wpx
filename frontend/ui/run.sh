#!/usr/bin/env bash

set -e
# set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
}

function backend() {
	cd ~/work/projects/desktop/track/profile
	dev.rust
	export RUST_BACKTRACE=1
	cargo test -- --nocapture
	sleep 1
	cargo build
	rm -f /tmp/*.svg /tmp/*.pdf
	rm /tmp/document.pdf 
	time cargo run 
	ls -l /tmp/*.pdf
	#nohup atril /tmp/test.pdf &
	#sleep 3
}

function frontend() {
	cd ~/work/projects/desktop/track/profile/frontend/ui/
	dev.flutter-rust
	cp ~/work/projects/desktop/track/profile/backend/data/blackforest.gpx /tmp/track.gpx
	flutter run --device-id Linux
}

function main() {
	# backend
	frontend
}

init
if ! main "$@"; then
	echo failed
else
	echo good
fi

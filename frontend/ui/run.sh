#!/usr/bin/env bash

set -e
# set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
}

function backend() {
	cd ~/projects/sandbox/desktop/track/profile
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
	cd ~/projects/sandbox/desktop/track/profile/frontend/ui/
	dev.flutter-rust
	cp ~/projects/sandbox/desktop/track/profile/backend/data/blackforest.gpx /tmp/track.gpx
	flutter run --device-id Linux
}

function run-web() {
	cd ~/projects/sandbox/desktop/track/profile/frontend/ui
	dev.flutter-rust
	rm -Rf /tmp/build.d
	mv build /tmp/build.d
	rustup target add wasm32-unknown-unknown
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen build-web
	flutter build web --debug
	mkdir -p build/web/pkg/
	cp web/pkg/* build/web/pkg/
	cp server.py build/web/
	cd build/web
	PORT=8123
	killall python3
	sleep 1
	python3 server.py http localhost &
	sleep 1
	firefox "http://localhost:8123/"
}

function main() {
	# backend
	# frontend
	run-web
}

init
if ! main "$@"; then
	echo failed
else
	echo good
fi

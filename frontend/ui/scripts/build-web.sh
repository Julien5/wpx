#!/usr/bin/env bash

set -e
# set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
}

function build() {
	cd ~/work/projects/desktop/track/profile/frontend/ui
	dev.flutter-rust
	rm -Rf /tmp/build.d
	mv build /tmp/build.d
	rustup target add wasm32-unknown-unknown
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen build-web
	flutter build web --debug
	mkdir -p build/web/pkg/
	cp web/pkg/* build/web/pkg/
}

function main() {
	build
	./scripts/touch-version.sh
	tar -C build -zcvf /tmp/web.tgz web

	# upload
	DOMAIN=vps-e637d6c5.vps.ovh.net
	scp -i ~/.ssh/ovh/id /tmp/web.tgz debian@${DOMAIN}:/tmp/
}

init
if ! main "$@"; then
	echo failed
else
	echo good
fi

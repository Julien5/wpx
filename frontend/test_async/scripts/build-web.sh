#!/usr/bin/env bash

set -e

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
}

function build() {
	SRC=$HOME/projects/sandbox/desktop/track/profile/frontend/test_async
	cd ${SRC}
	dev.flutter-rust
	set -x
	dos2unix pubspec.yaml
	rm -Rf /tmp/build.d
	mv build /tmp/build.d
	rustup target add wasm32-unknown-unknown
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen generate
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen build-web 
	flutter build web
	mkdir -p build/web/pkg/
	cp -rv $(find /opt/flutter/ -name "flutter.js.map") build/web/
	cp -rv web/pkg/* build/web/pkg/
	tar -zcf /tmp/web.tgz build/web
}
 
function main() {
    build
	./tools/start-ovh.sh 
}

init
if ! main "$@"; then
	echo failed
else
	echo good
fi

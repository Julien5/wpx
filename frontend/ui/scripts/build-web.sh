#!/usr/bin/env bash

set -e
set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
}

function build() {
	cd ~/work/projects/desktop/track/profile/frontend/ui
	dev.flutter-rust
	dos2unix pubspec.yaml
	echo "incrementing build version..."
	perl -i -pe 's/^(version:\s+\d+\.\d+\.)(\d+)\+(\d+)$/$1.($2)."+".($3+1)/e' pubspec.yaml
	version=$(grep ^version pubspec.yaml | cut -f2 -d":" | tr -d " ")
	rm -Rf /tmp/build.d
	mv build /tmp/build.d
	rustup target add wasm32-unknown-unknown
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen build-web
	flutter build web --release --build-name=${version}
	mkdir -p build/web/pkg/
	cp web/pkg/* build/web/pkg/
}

function main() {
	build
	./scripts/touch-version.sh

	if [ "$1" = "deploy" ]; then
		tar -zcf /tmp/web.tgz build/web
		# upload
		DOMAIN=vps-e637d6c5.vps.ovh.net
		scp -i ~/.ssh/ovh/id scripts/start-ovh.sh scripts/server.py /tmp/web.tgz debian@${DOMAIN}:/tmp/
		ssh -i ~/.ssh/ovh/id debian@${DOMAIN} "chmod +x /tmp/start-ovh.sh; /tmp/start-ovh.sh"
	fi
}

init
if ! main "$@"; then
	echo failed
else
	echo good
fi

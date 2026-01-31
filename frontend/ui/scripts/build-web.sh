#!/usr/bin/env bash

set -e

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
	set -x
}

function parse-arguments() {
	RELEASE=
	TARBALL=/tmp/webapp.tgz
	while [[ $# -gt 0 ]]; do
		case $1 in
			--tarball)
				TARBALL="$2"
				shift 
				shift
				;;
			--release)
				RELEASE="--release"
				shift 
				;;
			-*|--*)
				echo "Unknown option $1"
				exit 1
				;;
		esac
	done
	echo RELEASE="${RELEASE}"
	echo TARBALL="${TARBALL}"
}

function build() {
	SRC=$HOME/projects/wpx/frontend/ui
	cd ${SRC}
	dev.flutter-rust
	dos2unix pubspec.yaml
	echo "incrementing build version..."
	perl -i -pe 's/^(version:\s+\d+\.\d+\.)(\d+)\+(\d+)$/$1.($2)."+".($3+1)/e' pubspec.yaml
	version=$(grep ^version pubspec.yaml | cut -f2 -d":" | tr -d " ")
	#rm -Rf /tmp/build.d
	#mv build /tmp/build.d
	# deep clean
	find . -name "*rust*lib*wasm" -print -delete
	# prevent build errors on subsequent native builds
	mkdir -p build/native_assets/linux
	rustup target add wasm32-unknown-unknown
	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen generate
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen build-web ${RELEASE}
	flutter build web ${RELEASE} --pwa-strategy=none --build-name=${version}
	mkdir -p build/web/pkg/
	cp -Rv $(find /opt/flutter/ -name "flutter.js.map") build/web/
	cp -Rf web/pkg/* build/web/pkg/
	tar -zcf ${TARBALL} build/web
}
 
function main() {
	build 
}

init
parse-arguments "$@"
main "$@"

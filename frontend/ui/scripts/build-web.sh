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

function rust-build-web() {
	# initial error was:
	#   panicked at .../flutter_rust_bridge-2.12.0/src/third_party/wasm_bindgen/worker_pool.rs:260:43:
	#   fail to create WorkerPool: JsValue(DataCloneError: WebAssembly.Memory object could not be cloned.
	
	# then googling pointed to
	#   https://github.com/fzyzcjy/flutter_rust_bridge/issues/2914
	# which gives this workaround, up to the __heap_base and __data_end flags.
	
	# then got error
	#   error: failed to prepare module for threading
	#   Caused by:
    #     failed to find `__heap_base` for injecting thread id
	#
	# Gemini solved the problem, telling me to add the __heap_base and __data_end flags.
	# Kind of black magic.
	/opt/rust/cargo/bin/flutter_rust_bridge_codegen build-web ${RELEASE} --wasm-pack-rustflags "-Ctarget-feature=+atomics -Clink-args=--shared-memory -Clink-args=--max-memory=1073741824 -Clink-args=--import-memory -Clink-args=--export=__wasm_init_tls -Clink-args=--export=__tls_size -Clink-args=--export=__tls_align -Clink-args=--export=__tls_base -Clink-arg=--export=__heap_base -Clink-arg=--export=__data_end"
}

function build() {
	SRC=$HOME/projects/wpx/frontend/ui
	cd ${SRC}
	dev.flutter
	dev.rust
	dos2unix pubspec.yaml
	echo "incrementing build version..."
	perl -i -pe 's/^(version:\s+\d+\.\d+\.)(\d+)\+(\d+)$/$1.($2)."+".($3+1)/e' pubspec.yaml
	version=$(grep ^version pubspec.yaml | cut -f2 -d":" | tr -d " ")
	# NOBUILD=1 # to test the webapp loading page, we dont want to rebuild everything.
	if [ -z ${NOBUILD} ]; then
		# deep clean
		find . -name "*rust*lib*wasm" -print -delete
		# prevent build errors on subsequent native builds
		mkdir -p build/native_assets/linux
		rustup target add wasm32-unknown-unknown
		rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
		/opt/rust/cargo/bin/flutter_rust_bridge_codegen generate
		# /opt/rust/cargo/bin/flutter_rust_bridge_codegen build-web ${RELEASE}
		rust-build-web
		if [ "${RELEASE}" = "--release" ]; then
			flutter build web ${RELEASE} --pwa-strategy=none --build-name=${version}
		else
			flutter build web --debug --pwa-strategy=none --build-name=${version}
		fi
	else
		if [ -f ${TARBALL} ]; then
			echo reusing ${TARBALL}
			return
		fi
		cp web/*.* build/web/
		json=$(cat <<EOF
{"app_name":"wpx","version":"${version}","build_number":"19","package_name":"wpx"}
EOF
			)
		echo ${json} > build/web/version.json
		sed -i "s/\$FLUTTER_BASE_HREF/\/${version}\//g" build/web/index.html
	fi
	mkdir -p build/web/pkg/
	cp -Rv $(find /opt/flutter/ -name "flutter.js.map") build/web/
	cp -v web/*.png build/web/
	cp -Rvf web/pkg/* build/web/pkg/
	tar -zcf ${TARBALL} build/web
}

function main() {
	build 
}

init
parse-arguments "$@"
main "$@"

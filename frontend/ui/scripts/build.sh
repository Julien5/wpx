#!/usr/bin/env bash

set -euo pipefail
# set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	PROJECTDIR=$(realpath ${SCRIPTDIR}/..)
}

function parse-arguments() {
	TARGET=
	MODE=
	RUN=
	APP=
	while [[ $# -gt 0 ]]; do
		case $1 in
			--target)
				TARGET=$2
				shift
				shift
				;;
			--mode)
				MODE=$2
				shift
				shift
				;;
			--build-app)
				APP=1
				shift
				;;
			--run)
				RUN=1
				APP=1
				shift 
				;;
			-*|--*)
				echo "Unknown option $1"
				exit 1
				;;
		esac
	done
	if [ -z ${TARGET} ] || [ -z ${MODE} ]; then
		echo "build.sh --target TARGET --mode MODE [--build-app] [--run]"
		echo "TARGET: android-arm64 android-x64 linux"
		echo "MODE: debug release"
		echo
		echo "TODO: add web support"
		return 1
	fi
}

function load-toolchain() {
	source ~/projects/config/profile/profile.rust.sh
	source ~/projects/config/profile/profile.flutter.sh
	case "$TARGET" in
		android-*)
			source ~/projects/config/profile/profile.android.sh
			;;
	esac
}

function build-rust-worker() {
	case "$TARGET" in
		android-arm64)
			# device
			cargo ndk -t arm64-v8a -o  ${PROJECTDIR}/android/app/src/main/jniLibs build
			;;
		android-x64)
			# emulator 
			cargo ndk -t x86_64  -o ${PROJECTDIR}/android/app/src/main/jniLibs build
			;;
		linux)
			cargo build
			DESTDIR=${PROJECTDIR}/build/rust
			mkdir -p ${DESTDIR}
			cp -v ${CARGO_TARGET_DIR}/${MODE}/librust_lib_ui.so ${DESTDIR}
			;;
		*)
			echo "unsupported target: ${TARGET}"
			exit 1
			;;
	esac
}

function build-rust() {
	cd rust
	build-rust-worker
	cd ..
}

function build-flutter() {
	local flutter_opt=""
	if [ "$MODE" = "release" ]; then
		flutter_opt="--release"
	fi
	if [[ "$TARGET" = *"android"* ]]; then
		if [ "$MODE" = "debug" ]; then
			flutter_opt="--debug"
		fi
	fi
	set -x
	# needed by cmake for the linux target
	case "$TARGET" in
		android-*)
			flutter build apk ${flutter_opt}
			;;
		linux)
			flutter build linux ${flutter_opt}
			;;
	esac
}

function run() {
	case "$TARGET" in
		android-arm64)
			flutter run -d 25131JEGR02219 --${MODE}
			;;
		android-x64)
			# Note: -gpu host necessary on debian to prevent emulator crash
			# in unclear circumstances.
			# see blog/2026.07.24/main.md
			# flutter run -d emulator-pixel-6a --${MODE}
			flutter run -d emulator-pixel6a-root --${MODE} 
			;;
		linux)
			flutter run linux --${MODE}
			;;
	esac
}

function main() {
	load-toolchain
	build-rust
	if [ ! -z "${APP}" ]; then
		if [ -z "${RUN}" ]; then
			# flutter run rebuilds the app anyway
			build-flutter
		fi
	fi
	if [ ! -z "${RUN}" ]; then
		run
	fi
}

init 
if parse-arguments "$@"; then
	main 
fi


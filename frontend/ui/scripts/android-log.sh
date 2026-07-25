#!/usr/bin/env bash

set -euo pipefail
# set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	PROJECTDIR=$(realpath ${SCRIPTDIR}/..)
}

function parse-arguments() {
	TARGET=
	ALL=
	KERNEL=
	while [[ $# -gt 0 ]]; do
		case $1 in
			--target)
				TARGET=$2
				shift
				shift
				;;
			--all)
				ALL=1
				shift 
				;;
			--kernel)
				KERNEL=1
				shift 
				;;
			-*|--*)
				echo "Unknown option $1"
				exit 1
				;;
		esac
	done
	if [ -z ${TARGET} ]; then
		echo "$0 --target TARGET"
		echo "TARGET: android-arm64 android-x64"
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


function kernel() {
	load-toolchain
	case "$TARGET" in
		android-arm64)
			adb -s 25131JEGR02219 -e shell "su 0 dmesg -w" 
			;;
		android-x64)
			# adb -s emulator-5554 -e shell "su 0 dmesg -w"
			adb -s emulator-5554 -e shell "su 0 dmesg -w" 
			;;
		*)
			echo unsupported $TARGET
			;;
	esac
}

function logcat() {
	load-toolchain
	case "$TARGET" in
		android-arm64)
			adb -s 25131JEGR02219 logcat  -b all -v threadtime,usec *:V 
			;;
		android-x64)
			adb -s emulator-5554 logcat  -b all -v threadtime,usec *:V 
			;;
		*)
			echo unsupported $TARGET
			;;
	esac
}

function main() {
	load-toolchain
	if [  "$KERNEL" = 1 ]; then
		kernel
	else
		if [ "$ALL" = 1 ]; then
			logcat 
		else
			logcat | grep com.example.ui/com.example.ui.MainActivity
		fi
	fi
}


init 
if parse-arguments "$@"; then
	main
fi


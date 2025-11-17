#!/usr/bin/env bash

set -e
# set -x

TYPST=/opt/typst/typst-x86_64-unknown-linux-musl/typst

function segment-length() {
	local file=$1
	shift
	if [[ "${file}" = *jerome* ]]; then
		echo 35
		return
	fi
	echo 1010
}

function segment-overlap() {
	local file=$1
	shift
	if [[ "${file}" = *jerome* ]]; then
		echo 5
		return
	fi
	echo 10
}

function pdf() {
	set -x
	echo "args:"$@
	cmd=run
	options=
	file=data/blackforest.gpx
	while [ $# -gt 0 ]; do
		case "$1" in
			*.gpx)
				file=$1
				shift
				;;
			flamegraph)
				cmd=flamegraph
				shift
				;;
			main-test)
				options="--main-test true"
				file=data/ref/berlin.gpx
				shift
				;;
			*)
				echo unknown option "$1"
				exit 1
		esac
	done
	echo make pdf
	export RUST_LOG=trace
	cargo build # --release
	export CARGO_PROFILE_RELEASE_DEBUG=true
	rm -Rf /tmp/wpx
	mkdir /tmp/wpx
	time cargo ${cmd} -- \
		  --output-directory /tmp/wpx/ \
		  --debug true \
		  --step-elevation-gain 100 \
		  --segment-length $(segment-length ${file}) \
		  --segment-overlap $(segment-overlap ${file}) \
		  --profile-max-area-ratio 0.07 \
		  --map-max-area-ratio 0.12 \
		  ${options} \
		  "${file}"
	${TYPST} compile /tmp/document.typst
	echo xdg-open /tmp/document.pdf 
}

function filter-log {
	local filename=$1
	shift
	# Finished `dev` profile
	grep -v "Finished \`dev\` profile" ${filename} > /tmp/tmp
	mv /tmp/tmp ${filename}
}

function unit-tests() {
	export RUST_LOG=trace
	# export RUST_BACKTRACE=1
	cargo test $@ -- --nocapture
}

function main() {
	if [ ! -z "$1" ] && [ $1 = "unit-tests" ]; then
		shift 
		unit-tests "$@"
		return;
	else
		export RUST_BACKTRACE=1
		2>&1 pdf "$@"
	fi
	# run-test
}


main "$@"

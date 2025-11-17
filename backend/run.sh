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
	echo "args:"$@
	file=data/blackforest.gpx
	cmd=run
	if [ ! -z "$1" ] && [ "$1" = "flamegraph" ]; then
		cmd=flamegraph
		shift
	fi
	set -x
	last_arg="${@: -1}"
	if [ ! -z "$last_arg" ] && [ -f "$last_arg" ]; then
		file="$last_arg"
		set -- "${@:1:$(($#-1))}"
	fi
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
		  "$@" \
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
		pdf "$@"
	fi
	# run-test
}


main "$@"

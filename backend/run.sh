#!/usr/bin/env bash

set -e
# set -x

SCRIPTDIR=$(dirname $(realpath "$0"))

function segment-length() {
	local file=$1
	shift
	if [[ "${file}" = *jerome* ]]; then
		echo 28
		return
	fi
	echo 110
}

function segment-overlap() {
	local file=$1
	shift
	if [[ "${file}" = *jerome* ]]; then
		echo 3
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
	mode=""
	while [ $# -gt 0 ]; do
		case "$1" in
			*.gpx)
				file=$1
				shift
				;;
			flamegraph)
				cmd="flamegraph --no-inline"
				#cmd="flamegraph"
				shift
				;;
			--release)
				mode="--release"
				shift
				;;
			performance-test)
				options="--performance-test true"
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
	cargo build ${mode}
	export CARGO_PROFILE_RELEASE_DEBUG=true
	rm -Rf /tmp/wpx
	mkdir /tmp/wpx
	time cargo ${cmd} ${mode} --bin wpx -- \
		 --output /tmp/wpx/route.zip \
		 --start-time "2026-04-10T00:00:00" \
		 --speed "KMH-20.0" \
		 --kinds Controls,CutOff,GPXWaypoints,Cities,Villages,Mountains \
		 --debug true \
		 --step-elevation-gain 100 \
		 --segment-length $(segment-length ${file}) \
		 --segment-overlap $(segment-overlap ${file}) \
		 ${options} \
		 "${file}"
	
	unzip -o /tmp/wpx/route.zip -d /tmp route.pdf 
	echo xdg-open /tmp/route.pdf 
}

function cli() {
	set -x
	cargo run -- \
		  --output /tmp/foo.zip \
		  --step-elevation-gain 50 \
		  --segment-length 110 \
		  --segment-overlap 10 \
		  --start-time "2026-01-10T20:00:00" \
		  --speed 15 \
		  ~/tours/self/2024/05/gpx/with-waypoints/{jura,foret-noire}.gpx

	
}

function filter-log {
	local filename=$1
	shift
	# Finished `dev` profile
	grep -v "Finished \`dev\` profile" ${filename} > /tmp/tmp
	mv /tmp/tmp ${filename}
}

function unit-tests() {
	# export RUST_LOG=trace
	export RUST_BACKTRACE=1
	rm -Rf /tmp/graphs/
	2>&1 cargo test $@ -- --nocapture # --test-threads=1
}

function performance-test() {
	rm -Rf /tmp/*.svg /tmp/wpx
	export RUST_LOG=trace
	export RUST_BACKTRACE=1
	cargo flamegraph --bin wpx --no-inline -- --performance-test true --output /tmp/route.zip --debug false --step-elevation-gain 500 data/ref/pbp2023.gpx 
}

function render-graph() {
	rm -Rf /tmp/*.svg /tmp/wpx /tmp/graphs/
	export RUST_LOG=trace
	export RUST_BACKTRACE=1
	2>&1 cargo run -- --render-graph true \
		 --start-time "2026-04-12T00:00:00" \
		 --speed ACP \
		 --output /tmp/wpx.zip \
		 --debug true \
		 data/PBP-simple.gpx 
}

function render-wheel() {
	export RUST_LOG=trace
	export RUST_BACKTRACE=1
	cargo run -- --render-wheel true "$@"
}

function main() {
	if [ ! -z "$1" ]; then
		if [ $1 = "unit-tests" ]; then
			shift 
			unit-tests "$@"
			return;
		elif [ $1 = "render-wheel" ]; then
			shift 
			render-wheel "$@"
			return;
		elif [ $1 = "performance-test" ]; then
			shift 
			performance-test "$@"
			return;
		elif [ $1 = "render-graph" ]; then
			shift 
			render-graph "$@"
			return;
		elif [ $1 = "cli" ]; then
			shift 
			2>&1 cli "$@"
			return;
		elif [ $1 = "pdf" ]; then
			shift 
			2>&1 pdf "$@"
			return;
		fi
	else
		export RUST_BACKTRACE=1
		2>&1 pdf "$@"
	fi
	# run-test
}


main "$@"

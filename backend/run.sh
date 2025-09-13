#!/usr/bin/env bash

set -e
# set -x

TYPST=/opt/typst/typst-x86_64-unknown-linux-musl/typst

function pdf() {
	echo "args:"$@
	file=data/blackforest.gpx
	if [ -f "$1" ]; then
	   file="$1"
	   shift
	fi
	echo make pdf
	export RUST_LOG=debug
	time cargo run -- \
		  --output-directory /tmp/ \
		  --debug true \
		  --interval-length 100 \
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

function run-test() {
	rm -f /tmp/out.*.log
	cargo build
	for n in $(seq 1 5); do
		echo n=${n}
		rm -f /tmp/map-*.svg
		cargo run -- \
			  --output-directory /tmp/ \
			  --debug true \
			  --interval-length 180 \
			  data/ref/roland.gpx &> /tmp/out.${n}.log
		filter-log /tmp/out.${n}.log
		mv /tmp/document.typst /tmp/document.${n}.typst
		mv /tmp/map-0.svg /tmp/map.${n}.svg
		${TYPST} compile /tmp/document.${n}.typst 
	done
	md5sum /tmp/out.*.log
	md5sum /tmp/map.*.svg 
}

function main() {
	export RUST_BACKTRACE=1
	rm -f /tmp/*.{svg,pdf,gpx,typ} /tmp/document.*
	pdf "$@"
	# run-test
}


main "$@"

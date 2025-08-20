#!/usr/bin/env bash

set -e
set -x

function pdf() {
	echo make pdf
	cargo run -- \
		  --output-directory /tmp/ \
		  --debug true \
		  data/blackforest.gpx
	TYPST=/opt/typst/typst-x86_64-unknown-linux-musl/typst
	${TYPST} compile /tmp/document.typst
	echo xdg-open /tmp/document.pdf 
}

function exp() {
	echo make exp
	cargo run -- \
		  --output-directory /tmp/ \
		  --experiment-labels true \
		  --debug true \
		  data/blackforest.gpx
}

function main() {
	export RUST_BACKTRACE=1
	rm -f /tmp/*.{svg,pdf,gpx,typ} /tmp/document.*
	pdf
	# exp
	ls /tmp/*.svg
	ls /tmp/*.pdf
}


main "$@"

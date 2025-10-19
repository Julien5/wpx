#!/usr/bin/env bash

function main() {
	cargo build --release
	scp ${CARGO_TARGET_DIR}/release/server debian@vps-e637d6c5.vps.ovh.net:/tmp/
}

main

#!/usr/bin/env bash

# 2963  rustup target add x86_64-unknown-linux-musl
# 2965  sudo apt install musl-tools
# 2966  cargo build --release --target x86_64-unknown-linux-musl

function main() {
	cargo build --release --target x86_64-unknown-linux-musl
	scp ${CARGO_TARGET_DIR}/x86_64-unknown-linux-musl/release/server \
		debian@vps-e637d6c5.vps.ovh.net:/tmp/server
}

main

#!/usr/bin/env bash

set -e
set -x

echo "OK"

MINISERVE=/home/julien/delme/rust-targets/debug/miniserve
cd /home/julien/projects/sandbox/desktop/track/profile/frontend/ui/build/web

${MINISERVE} \
	--tls-cert /tmp/localhost.cert  \
	--tls-key /tmp/localhost.key \
	--index index.html \
	--header "Cross-Origin-Opener-Policy:same-origin" \
	--header "Cross-Origin-Embedder-Policy:require-corp" \
	--port 8123 \
	--verbose

	.

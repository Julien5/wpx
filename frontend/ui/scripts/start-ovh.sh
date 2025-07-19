#!/usr/bin/env bash

set -e
set -x

if [ -f /tmp/web.tgz ]; then
	rm -Rf web
	tar xvf /tmp/web.tgz
fi

cd build/web;

killall miniserve || true
sleep 1

function runminiserve() {
	MINISERVE=/tmp/miniserve
	DOMAIN=vps-e637d6c5.vps.ovh.net
	nohup ${MINISERVE} \
	--tls-cert /tmp/${DOMAIN}.cert  \
	--tls-key /tmp/${DOMAIN}.key \
	--index index.html \
	--header "Cross-Origin-Opener-Policy:same-origin" \
	--header "Cross-Origin-Embedder-Policy:require-corp" \
	--port 8123 \
	--verbose \
	"$@" \
	&> /tmp/server.log 
}

runminiserve &
sleep 1
echo ok



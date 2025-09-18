#!/usr/bin/env bash

set -e
set -x

function init() {
	cd $HOME
}

# https://lukasnevosad.medium.com/our-flutter-web-strategy-for-deferred-loading-instant-updates-happy-users-45ed90a7727c
function make-dist() {
	local DISTDIR=$1
	shift

	echo unpacking tarball
	if [ ! -f /tmp/web.tgz ]; then
		echo could not find /tmp/web.tgz
		echo please "tar -zcf /tmp/web.tgz build/web"
		echo "(and updload)"
		return 1
	fi
	
	rm -Rf /tmp/build/
	tar -C /tmp/ -xf /tmp/web.tgz
	local BUILDDIR=/tmp/build/web
	if [ ! -d ${BUILDDIR} ]; then
		echo could not find "${BUILDDIR}" after unpacking
		return 
	fi
	
	# Get version from version.json
	local VERSION=$(sed -n 's|.*"version":"\([^"]*\)".*|\1|p' "$BUILDDIR/version.json")
	echo "distributing version: $VERSION"
	mkdir -p "$DISTDIR"

	# <base href="/" />
	# <base href="/1.30.11+504102/" />
	rm -Rf "$DISTDIR/$VERSION"
	cp -Rf "$BUILDDIR" "$DISTDIR/$VERSION"
	mv "$DISTDIR/$VERSION/index.html" "$DISTDIR"
	cp "$DISTDIR/$VERSION/version.json" "$DISTDIR"
	sed -i "s|<base href=\"/\">|<base href=\"/$VERSION/\" />|g" "$DISTDIR/index.html"
	grep href "$DISTDIR/index.html"
}

function runminiserve() {
	local CARGO_TARGET_DIR=$HOME/delme/rust-targets
	for a in /tmp/miniserve ${CARGO_TARGET_DIR}/release/miniserve /opt/miniserve; do
		if [ -f ${a} ]; then
			MINISERVE=${a}
			break;
		fi
	done
	
	if [ -z "${MINISERVE}" ]; then
		echo could not find miniserve
		return 1
	fi
	
	DOMAIN=localhost
	if [ "$(hostname)" = vps-e637d6c5 ]; then
		DOMAIN=vps-e637d6c5.vps.ovh.net
	fi
	
	nohup ${MINISERVE} \
		  --tls-cert /tmp/${DOMAIN}.cert  \
		  --tls-key /tmp/${DOMAIN}.key \
		  --spa --index index.html \
		  --header "Cross-Origin-Opener-Policy:same-origin" \
		  --header "Cross-Origin-Embedder-Policy:require-corp" \
		  --header "Access-Control-Allow-Headers:*" \
		  --port 8123 \
		  --verbose \
		  "$@" \
		&> /tmp/server.log 
}

function main() {
	echo stop old server.. 
	killall miniserve || true
	sleep 1

	if ! make-dist /tmp/dist; then
		echo make-dist failed.
		echo moving on.
	fi
	cd /tmp/dist
	
	runminiserve &
	sleep 5
	echo ok
}

main "$@"

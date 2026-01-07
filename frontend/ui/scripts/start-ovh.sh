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

function runserver() {
	local CARGO_TARGET_DIR=$HOME/delme/rust-targets
	for a in /tmp/server \
			 ${CARGO_TARGET_DIR}/release/server; do
		if [ -f ${a} ]; then
			SERVER=${a}
			break;
		fi
	done
	
	if [ -z "${SERVER}" ]; then
		echo could not find server
		return 1
	fi
	
	DOMAIN=localhost
	if [ "$(hostname)" = vps-e637d6c5 ]; then
		DOMAIN=vps-e637d6c5.vps.ovh.net
	fi

	local PORT=$1
	shift
	local DIR=$1
	shift

	echo port: $PORT
	echo dir: $DIR
	set +e
	OLDPID=$(lsof -ti :${PORT})
	if [ ! -z "${OLDPID}" ]; then
		echo kill ${OLDPID}
		lsof -i :${PORT} || true
		kill ${OLDPID} || true
	fi
	echo continuing

	LOGDIR=$HOME/logs/${PORT}
	TIMESTAMP=$(date +%Y.%m.%d-%H.%M.%S)
	mkdir -p ${LOGDIR}
	
	nohup ${SERVER} \
		  --cert /tmp/${DOMAIN}.cert  \
		  --key /tmp/${DOMAIN}.key \
		  --port ${PORT} \
		  --directory ${DIR} \
		&> ${LOGDIR}/${TIMESTAMP}.log 
}

function main() {
	echo stop old server.. 
	sleep 1
	local MASTER=
	if [ "${1:-}" = "master" ]; then
		MASTER=1
	fi

	DIR=/tmp/dist-test
	PORT=8124
	if [ "${MASTER}" = "1" ]; then
		DIR=/tmp/dist
		PORT=8123
	fi

	if ! make-dist ${DIR}; then
		echo make-dist failed on ${DIR}
		echo moving on.
	fi
	cd ${DIR}
	
	# runminiserve &
	runserver ${PORT} ${DIR} &
	sleep 5
	echo ok
}

main "$@"

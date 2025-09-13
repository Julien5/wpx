#!/usr/bin/env bash

set -e
# set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
}

function make-certs() {
	cd /tmp/
	local DOMAIN=$1 # localhost or vps-e637d6c5.vps.ovh.net
	shift
	rm -vf /tmp/${DOMAIN}.*
	SUBJ="/C=XX/ST=Germany/L=Ingoldingen/O=JBO/OU=dev/CN=${DOMAIN}";
	openssl req -x509 -out ${DOMAIN}.cert \
			-keyout ${DOMAIN}.key \
			-newkey rsa:2048 -nodes -sha256  \
			-subj "${SUBJ}"
	ls /tmp/${DOMAIN}.*
	openssl x509 -noout -text -in /tmp/${DOMAIN}.cert  | grep Subject 
}

function main() {
	make-certs localhost
}

init
if ! main "$@"; then
	echo failed
else
	echo good
fi

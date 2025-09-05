#!/usr/bin/env bash

function dl-cities-worker() {
	local BBOX=$1
	shift
	local filename=$1
	shift
	curl 'https://overpass-api.de/api/interpreter' \
		 --compressed -X POST \
		 -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0' \
		 -H 'Accept: */*' \
		 -H 'Accept-Language: en-US,en;q=0.5' \
		 -H 'Accept-Encoding: gzip, deflate, br, zstd' \
		 -H 'Content-Type: application/x-www-form-urlencoded; charset=UTF-8' \
		 -H 'Origin: https://overpass-turbo.eu' \
		 -H 'Connection: keep-alive' \
		 -H 'Referer: https://overpass-turbo.eu/' \
		 -H 'Sec-Fetch-Dest: empty' \
		 -H 'Sec-Fetch-Mode: cors' \
		 -H 'Sec-Fetch-Site: cross-site' \
		 -H 'Priority: u=0' \
		 --data-raw "data=%2F*%0AThis+has+been+generated+by+the+overpass-turbo+wizard.%0AThe+original+search+was%3A%0A%E2%80%9Ccity%E2%80%9D%0A*%2F%0A%5Bout%3Ajson%5D%5Btimeout%3A25%5D%3B%0A%2F%2F+gather+results%0Anwr%5B%22place%22%3D%22town%22%5D${BBOX}%3B%0A%2F%2F+print+results%0Aout+geom%3B" \
		 --output ${filename}
}

function dl-cities-small() {
	local BBOX="(47.86385263046569%2C9.667968750000002%2C48.17432829641996%2C10.8050537109375)"
	dl-cities-worker "${BBOX}" data/cities-small.json
}

function dl-cities-south() {
	local BBOX="(47.39834920035926%2C6.306152343750001%2C49.85215166777001%2C15.402832031250002)"
	dl-cities-worker "${BBOX}" data/cities-south.json
}

function main() {
	dl-cities-small
	dl-cities-south
}

main

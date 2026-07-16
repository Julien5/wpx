#!/usr/bin/env bash

set -e
# set -x

function init() {
	SCRIPTDIR=$(realpath $(dirname $0))
	. $HOME/.profile
}

function package() {
	DIR=/tmp/x
	mkdir -p ${DIR}
	if [ ! -d ${DIR}/fit ]; then
		cd ${DIR}
		unzip -o ~/Downloads/GpxTracksFürJulien.zip
		mv GpxTracksFürJulien fit
	fi
    # rm -Rf  ${DIR}/csv
	if [ ! -d ${DIR}/csv ]; then 
		mkdir -p ${DIR}/csv
		mkdir -p ${DIR}/gpx
		find ${DIR}/fit -name "*.fit" | while read fit; do
			b=$(basename ${fit})
			gpsbabel -t -i garmin_fit -f ${fit} \
					 -o unicsv -F ${DIR}/csv/${b/fit/csv}
			gpsbabel -t -i garmin_fit -f ${fit} \
					 -o gpx -F ${DIR}/gpx/${b/fit/gpx}
		done
	fi
	dev.rust
	cd /home/julien/projects/wpx/tools/gps-preprocess/
	
	mkdir -p ${DIR}/processed
	A=2026-06-06-07-11-06
	cargo run -- \
		  --input ${DIR}/csv/${A}.csv \
		  --output ${DIR}/processed/${A}.csv
	
	mkdir -p ${DIR}/png/${A}
	plot=oversample
	gnuplot -e "filename='${DIR}/processed/${A}.csv'" \
			-e "pngfile='${DIR}/png/${A}/${plot}.png'" \
			-e "xrange='[1.03:1.05]'" \
			gnuplot/${plot}.gnuplot
	
	for plot in filter_outliers output slope smooth_speed; do
		gnuplot -e "filename='${DIR}/processed/${A}.csv'" \
				-e "pngfile='${DIR}/png/${A}/${plot}.png'" \
				-e "xrange='[5.9:6.1]'" \
				gnuplot/${plot}.gnuplot
	done
	
}

function main() {
	package
}

init
if ! main "$@"; then
	echo failed
else
	echo good
fi

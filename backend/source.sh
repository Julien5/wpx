#!/usr/bin/env bash

function sd() {
	local filename=$1
	shift
	local reffile=data/ref/$(basename ${filename})
	for a in ${filename} ${reffile}; do
		if [ ! -f "${a}" ]; then
			echo could not find "${a}"
			return 1;
		fi
	done 
	if ! diff -q ${filename} ${reffile}; then 
		firefox ${filename} ${reffile}
	else
		echo ${filename} ${reffile} are equal
	fi
}

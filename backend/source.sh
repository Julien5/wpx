#!/usr/bin/env bash

function sd() {
	local filename=$1
	shift
	local reffile=data/ref/$(basename ${filename})
	if ! diff -q ${filename} ${reffile}; then 
		firefox ${filename} ${reffile}
	else
		echo ${filename} ${reffile} are equal
	fi
}

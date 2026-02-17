#!/usr/bin/env bash

function sd() {
	local filename=$1
	shift
	local reffile=data/ref/$(basename ${filename})
	firefox ${filename} ${reffile}
}

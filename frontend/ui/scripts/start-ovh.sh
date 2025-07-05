#!/usr/bin/env bash

set -e
set -x

if [ -f /tmp/web.tgz ]; then
	rm -Rf web
	tar xvf /tmp/web.tgz
fi

cd build/web;

killall python3 || true
sleep 1

nohup python3 /tmp/server.py &> /tmp/server.log & 
sleep 1
echo ok



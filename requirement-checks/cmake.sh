#!/usr/bin/env bash

version="$(
	cmake --version |
		head -1 |
		sed -rn 's/.*cmake version ([0-9]+\.[0-9]+\.[0-9])( .*|$)/\1/p'
)"

IFS="." read -r major minor _patch <<<"$version"

if [[ -z $major ]]; then
	echo "error: failed to determine cmake version"
	exit 1
elif [[ $major -lt 3 ]]; then
	echo "error: your cmake version is too old: $version; required cmake >= 3.22.0"
	exit 1
fi

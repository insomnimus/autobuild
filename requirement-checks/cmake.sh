#!/usr/bin/env bash

version="$(
	cmake --version |
		head -1 |
		sed -rn 's/.*cmake version ([0-9]+\.[0-9]+\.[0-9])( .*|$)/\1/p'
)"

IFS="." read -r major minor _patch <<<"$version"

if [[ -z $major ]]; then
	echo  "error: failed to determine cmake version"
	echo "tip: run autobuild with --install-cmake to install an isolated compatible version"
	exit 1
elif [[ $major -gt 3 ]]; then
	echo  "error: your cmake version is too new: $version; required version is cmake < 4.0.0"
	echo  "sadly, cmake 4 is not compatible with some projects"
	echo "tip: run autobuild with --install-cmake to install an isolated compatible version"
	exit 1
elif [[ $major -lt 3 || ${minor:-0} -lt 22 ]]; then
	echo "error: your cmake version is too old: $version; required cmake >= 3.22.0"
	echo "tip: run autobuild with --install-cmake to install an isolated compatible version"
	exit 1
fi

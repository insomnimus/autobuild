#!/usr/bin/env bash

version="$(
	x86_64-w64-mingw32-gcc --version |
		head -n1 |
		sed -rn 's/.*x86_64-w64-mingw32-gcc \([^\)]+\) ([^ ]+).*/\1/p'
)"

if [[ -z $version ]]; then
	echo "error: failed to detect mingw64 gcc version (x86_64-w64-mingw32-gcc)"
	exit 1
fi

# Ubuntu issue of gcc prints versions like "10-win32"
major=""
if [[ $version == ?*-win32 ]]; then
	version="${version%-win32}"
fi

IFS=. read -r major _minor _patch <<<"$version"

if [[ ! $version =~ ^[0-9]+ ]]; then
	echo "error: failed to detect mingw64 gcc version (x86_64-w64-mingw32-gcc)"
	exit 1
elif [[ $major -lt 13 ]]; then
	echo "error: the mingw64 cross compiler toolchain version is too old: $version; required version >= 13"
	echo "you can obtain an up to date mingw64 toolchain from https://github.com/xpack-dev-tools/mingw-w64-gcc-xpack/"
	exit 1
fi

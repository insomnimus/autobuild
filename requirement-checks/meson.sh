#!/usr/bin/env bash

version="$(meson --version)"
if [[ -z $version ]]; then
	echo "error: failed to detect meson version"
	exit 1
elif [[ $version == 0.* ]]; then
	echo "error: meson version is too old: $version; required meson >= 1.0.0"
	echo "to get up to date meson: https://mesonbuild.com/Getting-meson.html"
	exit 1
fi

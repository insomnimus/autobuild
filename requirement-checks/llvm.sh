#!/usr/bin/env bash

major="$(
	clang --version 2>&1 |
		head -1 |
		sed -rn 's/.*clang version ([0-9]+)\.[0-9]+\.[0-9]+( .*|$)/\1/p'
)"

if [[ -z $major ]]; then
	echo "error: failed to detect clang version"
	exit 1
elif [[ $major -lt 16 ]]; then
	echo "error: the available clang version is too old: $major; required clang >= 16"
	distro="$(sed -rn 's/^ *ID="?([^"]+)"?/\1/p' /etc/os-release 2>/dev/null)"
	case "$distro" in
	ubuntu | debian)
		echo "to obtain up to date llvm packages visit https://apt.llvm.org"
		echo "^ you will have to create symbolic links such as /usr/local/bin/clang -> /usr/bin/clang-17; the version suffix is not recognized in autobuild"
		;;
	*) ;;
	esac
	exit 1
fi

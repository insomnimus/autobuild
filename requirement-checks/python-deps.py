#!/usr/bin/env python3

try:
	import packaging
except ModuleNotFoundError:
	import sys
	print("error: required python module 'packaging' is not installed; please install it with your package manager", file=sys.stderr)
	exit(1)

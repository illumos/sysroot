#!/bin/bash

MACH64=amd64

PROTO=$1
shift

if [[ -z "$PROTO" || ! -d "$PROTO" ]]; then
	exit 1
fi

BASENAME=$1
shift

for linkname in "$@"; do
	for p in "/usr/lib/$linkname" "/usr/lib/$MACH64/$linkname"; do
		printf '    linking: %s -> %s\n' "$p" "$BASENAME"
		rm -f "$PROTO/$p"
		ln -s "$BASENAME" "$PROTO/$p"
	done
done

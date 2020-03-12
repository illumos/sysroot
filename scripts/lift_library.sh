#!/bin/bash

MACH64=amd64

PROTO=$1
shift

if [[ -z "$PROTO" || ! -d "$PROTO" ]]; then
	exit 1
fi

BASENAME=$1
shift

if [[ -z "$BASENAME" || -L "/usr/lib/$BASENAME" || ! -f "/usr/lib/$BASENAME" ||
    -L "/usr/lib/$MACH64/$BASENAME" || ! -f "/usr/lib/$MACH64/$BASENAME" ]]; then
	printf 'ERROR: invalid base name\n' >&2
	exit 2
fi

for p in "/usr/lib/$BASENAME" "/usr/lib/$MACH64/$BASENAME"; do
	printf '    lifting: %s\n' "$p"
	rm -f "$PROTO/$p"
	cp "$p" "$PROTO/$p"
done

for linkname in "$@"; do
	for p in "/usr/lib/$linkname" "/usr/lib/$MACH64/$linkname"; do
		printf '    linking: %s -> %s\n' "$p" "$BASENAME"
		rm -f "$PROTO/$p"
		ln -s "$BASENAME" "$PROTO/$p"
	done
done

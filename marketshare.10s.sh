#!/bin/bash
SCRIPTDIR=$(dirname "$(readlink -f "$0")")
EXECUTABLE=${SCRIPTDIR}/target/release/marketshare

if [ ! -f "$EXECUTABLE" ]; then
	pushd ${SCRIPTDIR}
	cargo build --release
	popd
fi

$EXECUTABLE

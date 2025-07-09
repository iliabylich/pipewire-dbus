#!/usr/bin/env bash

set -euo pipefail

BUILDTYPE="$1"
SOURCE_ROOT="$2"
OUTPUT="$3"

case "$BUILDTYPE" in
    "release")
        CARGO_OPT="--release"
        ;;
    "debug")
        CARGO_OPT=""
        ;;
    *)
        echo "Usage: $0 [debug|release] <source-root> <output>"
        exit 1
        ;;
esac

cargo build $CARGO_OPT --manifest-path "$SOURCE_ROOT/Cargo.toml"
cp "$SOURCE_ROOT/target/$BUILDTYPE/pipewire-dbus" "$OUTPUT"

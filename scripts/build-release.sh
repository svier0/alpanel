#!/bin/sh
set -e

cd "$(dirname "$0")/../cli"
cargo build --release

for target in x86_64-unknown-linux-musl aarch64-unknown-linux-musl; do
    dir="cli/target/$target/release"
    out="cli/target/$target.tar.gz"
    tar -czf "$out" -C "$dir" alp
    echo "Packaged: $out"
done

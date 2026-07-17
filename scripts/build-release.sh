#!/bin/sh
set -e

cd "$(dirname "$0")/.."
mkdir -p releases
cd backend

version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
cargo build --release

targets=$(sed -n '/^\[build\]/,/^\[/p' .cargo/config.toml | grep '^target = ' | sed 's/.*\[//; s/\].*//; s/"//g; s/,//g')

for target in $targets; do
    dir="target/$target/release"
    out="../releases/alpanel-$version-$target.tar.gz"
    tar -czf "$out" -C "$dir" alpanel
    echo "Packaged: $out"
done

# 前端单独打包为 dist-<version>.tar.gz
cd ../frontend
bun run build
cd ..
dist_out="releases/dist-$version.tar.gz"
tar -czf "$dist_out" -C frontend/dist .
echo "Packaged: $dist_out"

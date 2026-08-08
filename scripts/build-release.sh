#!/bin/sh
set -e

cd "$(dirname "$0")/.."
mkdir -p releases

version=$(grep '^version = ' backend/Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# 1. 编译前端
cd frontend
pnpm run build
cd ..

# 2. 复制前端 dist/* 完整到 panel/dist/*
panel_dir="$PWD/panel"
mkdir -p "$panel_dir/dist"
rm -rf "$panel_dir/dist/"*
cp -r frontend/dist/* "$panel_dir/dist/"
echo "Copied frontend dist -> panel/dist"

# 3. targets 从 .cargo/config.toml 读取
targets=$(sed -n '/^\[build\]/,/^\[/p' backend/.cargo/config.toml | grep '^target = ' | sed 's/.*\[//; s/\].*//; s/"//g; s/,//g')

# 4. for target: 编译后端 + 复制 alpanel + 打包 panel 整个目录
for target in $targets; do
    echo "Building backend for $target"
    (cd backend && cargo build --release --target "$target")

    cp "backend/target/$target/release/alpanel" "$panel_dir/alpanel"

    out="releases/alpanel-$version-$target.tar.gz"
    tar -czf "$out" -C "$panel_dir" .
    echo "Packaged: $out"
done

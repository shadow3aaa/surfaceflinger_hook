#!/bin/bash

set -e

rm -rf output/*
mkdir -p output

cd output

CFLAGS="
-Os -flto -fmerge-all-constants -fno-exceptions -fomit-frame-pointer -fshort-enums
-Wl,-O3,--lto-O3,--gc-sections,--as-needed,--icf=all,-z,norelro,--pack-dyn-relocs=android+relr
"

# inject
aarch64-linux-android-clang++ $CFLAGS ../inject/inject.cpp -o inject
aarch64-linux-android-strip inject

# libsurfaceflinger_hook.so
cargo b -r --target aarch64-linux-android
cp -f ../target/aarch64-linux-android/release/libsurfaceflinger_hook.so .

# package
cp -rf ../module/* .
rm -f ../surfaceflinger_hook.zip
zip -9 -rq ../surfaceflinger_hook.zip .

echo "Packaged as magisk module: $(realpath ../surfaceflinger_hook.zip)"

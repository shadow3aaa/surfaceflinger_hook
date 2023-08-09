#!/bin/bash

set -e

rm -rf output/*
mkdir -p output

cd output

CFLAGS="
-Os -flto -fmerge-all-constants -fno-exceptions -fomit-frame-pointer -fshort-enums
-Wl,-O3,--lto-O3,--gc-sections,--as-needed,--icf=all,-z,norelro,--pack-dyn-relocs=android+relr
"

aarch64-linux-android-clang++ $CFLAGS ../inject/inject.cpp -w -o inject
aarch64-linux-android-clang++ --shared ../hookLib/hooklib.cpp ../dobby/arm64-v8a/libdobby.a $CFLAGS -std=c++2b -lc++ -fPIC -llog -static-libstdc++ -o hookLib.so

aarch64-linux-android-strip inject

cp -rf ../module/* .
zip -9 -rq ../surfaceflinger_hook.zip .
echo "Packaged as magisk module: $(realpath ../surfaceflinger_hook.zip)"

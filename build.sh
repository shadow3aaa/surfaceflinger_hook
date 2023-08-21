#!/bin/bash

set -e

BASEDIR="$(pwd)"

# build inject
cd $BASEDIR/AndroidPtraceInject

rm -rf build
mkdir build
cd build

cmake ../Inject
make -j4

# build hook lib
cd $BASEDIR
cargo b -r --target aarch64-linux-android

# package
rm -rf $BASEDIR/output
mkdir -p $BASEDIR/output
cd $BASEDIR/output

cp -rf $BASEDIR/module/* .
cp -f $BASEDIR/AndroidPtraceInject/Inject/outputs/Inject ./inject
cp -f target/aarch64-linux-android/release/libsurfaceflinger_hook.so ./libsurfaceflinger_hook.so

rm -f $BASEDIR/surfaceflinger_hook.zip
zip -9 -rq $BASEDIR/surfaceflinger_hook.zip .

# print out put path
echo "Packaged as magisk module: $(realpath ../surfaceflinger_hook.zip)"

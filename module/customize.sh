SKIPUNZIP=0

# permission
chmod a+x $MODPATH/inject

if [ "$ARCH" != "arm64" ]; then
	ui_print "Only supports arm64 architecture"
	abort
fi

symbol=$(readelf -s /system/lib64/libsurfaceflinger.so |
	grep -i postComposition |
	grep -i SurfaceFlinger)

# no symbol that can be hooked was found
if [ "$symbol" = "" ]; then
	echo "未找到此设备的hook位点"
	abort
fi

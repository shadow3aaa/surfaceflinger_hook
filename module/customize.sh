SKIPUNZIP=0

# permission
chmod a+x $MODPATH/inject

# only support aarch64
if [ "$ARCH" != "arm64" ]; then
	ui_print "Only supports arm64 architecture"
	abort
fi

# scan symbols
local symbol_pre=$(readelf -s /system/lib64/libsurfaceflinger.so |
	grep preComposition |
	grep CompositionEngine |
	awk '{print $NF}')

local symbol_post=$(readelf -s /system/lib64/libsurfaceflinger.so |
	grep postComposition |
	grep SurfaceFlinger |
	awk '{print $NF}')

# no symbol that can be hooked was found
if [[ $symbol_pre == "" || $symbols_post == "" ]]; then
	abort
fi

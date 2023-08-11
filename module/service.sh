MODDIR=${0%/*}
TMP_DIR=/data/surfaceflinger_hook
SO=$TMP_DIR/cache/libsufaceflinger_hook.so

# wait for surfaceflinger start
until pidof surfaceflinger; do
	sleep 1s
done

set_available_symbol() {
	local symbol=$(readelf -s /system/lib64/libsurfaceflinger.so |
		grep -i postComposition |
		grep -i SurfaceFlinger)

	# no symbol that can be hooked was found
	if [ "$symbol" = "" ]; then
		touch $MODDIR/disable
		exit 1
	fi

	local symbol=$(awk '{print $NF}' <<<"$symbol")

	# surfaceflinger reads this file to know which function to hook
	echo $symbol >$TMP_DIR/cache/available_symbol
}

set_caches() {
	mkdir -p $TMP_DIR
	mkdir $TMP_DIR/cache
	cp -f $MODDIR/libsufaceflinger_hook.so $SO
}

set_permissions() {
    magiskpolicy --live "allow surfaceflinger * * *"
    chown -R system:graphics $TMP_DIR
	chmod -R 0644 $TMP_DIR
}

inject() {
	local pid=$(pidof surfaceflinger)

	# reserve time for something unexpected
	sleep 60s

	$MODDIR/inject -p $pid -so $SO -symbols hook_surfaceflinger
}

set_caches
find_available_symbol
set_permissions
inject

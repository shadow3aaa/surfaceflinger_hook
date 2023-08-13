MODDIR=${0%/*}
HOOK_DIR=/dev/surfaceflinger_hook
SO=$HOOK_DIR/libsurfaceflinger_hook.so

# wait for surfaceflinger start
until pidof surfaceflinger; do
	sleep 1s
done

# from magisk
set_perm() {
	chown $2:$3 $1 || return 1
	chmod $4 $1 || return 1
	local CON=$5
	[ -z $CON ] && CON=u:object_r:system_file:s0
	chcon $CON $1 || return 1
}

# from magisk
set_perm_recursive() {
	find $1 -type d 2>/dev/null | while read dir; do
		set_perm $dir $2 $3 $4 $6
	done
	find $1 -type f -o -type l 2>/dev/null | while read file; do
		set_perm $file $2 $3 $5 $6
	done
}

set_available_symbol() {
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
	if [ "$symbol_pre" = "" || "$symbols_post" = "" ]; then
		touch $MODDIR/disable
		exit 1
	fi

	# Rust src:
	# let pre_path = Path::new("symbol_preComposition");
	# let post_path = Path::new(HOOK_DIR).join("symbol_postComposition");
	echo $symbol_pre >$HOOK_DIR/symbol_preComposition
	echo $symbol_post >$HOOK_DIR/
}

set_dir() {
	mkdir -p $HOOK_DIR
	cp -f $MODDIR/libsurfaceflinger_hook.so $SO
}

set_permissions() {
	magiskpolicy --live "allow surfaceflinger * * *"
	set_perm_recursive $HOOK_DIR graphics graphics 0644
	set_perm $HOOK_DIR graphics graphics 0777
}

inject() {
	local pid=$(pidof surfaceflinger)

	# reserve time for something unexpected
	sleep 60s

	$MODDIR/inject -p $pid -so $SO -symbols hook_surfaceflinger
	rm $SO
}

set_dir
set_available_symbol
set_permissions
inject

MODDIR=${0%/*}
TMP_DIR=/dev/surfaceflinger_hook
SO=$TMP_DIR/libsurfaceflinger_hook.so

# wait for surfaceflinger start
until pidof surfaceflinger; do
	sleep 1s
done

set_perm() {
	chown $2:$3 $1 || return 1
	chmod $4 $1 || return 1
	local CON=$5
	[ -z $CON ] && CON=u:object_r:system_file:s0
	chcon $CON $1 || return 1
}

set_perm_recursive() {
	find $1 -type d 2>/dev/null | while read dir; do
		set_perm $dir $2 $3 $4 $6
	done
	find $1 -type f -o -type l 2>/dev/null | while read file; do
		set_perm $file $2 $3 $5 $6
	done
}

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
	echo $symbol >$TMP_DIR/available_symbol
}

set_dir() {
	mkdir -p $TMP_DIR
	cp -f $MODDIR/libsurfaceflinger_hook.so $SO
}

set_permissions() {
	magiskpolicy --live "allow surfaceflinger * * *"
	set_perm_recursive $TMP_DIR graphics graphics 0644
	set_perm $TMP_DIR graphics graphics 0777
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

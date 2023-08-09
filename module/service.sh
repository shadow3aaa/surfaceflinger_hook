MODDIR=${0%/*}

# wait for lmkd start
until pidof surfaceflinger; do
	sleep 1s
done

$MODDIR/inject -p $(pidof surfaceflinger) -so $(realpath $MODDIR/hookLib.so) -symbols hook_surfaceflinger

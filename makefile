.DEFAULT_GOAL := all

.PHONY: all
all: inject hook_lib package

.PHONY: clean
clean:
	@cargo clean
	@cd AndroidPtraceInject && cargo clean

.PHONY: inject
inject:
	@echo "Building inject…"
	@cd AndroidPtraceInject; \
	cargo b -r --target aarch64-linux-android
	@echo "Built inject(bin) successfully"

.PHONY: hook_lib
hook_lib:
	@echo "Building hook lib…"
	@cargo b -r --target aarch64-linux-android
	@echo "Built surfaceflinger_hook(lib) successfully"
	
.PHONY: package
package: inject hook_lib
	@rm -rf output/*
	@mkdir -p output

	@cp -rf scripts/* output/
	@cp -f AndroidPtraceInject/target/aarch64-linux-android/release/inject output/
	@cp -f target/aarch64-linux-android/release/libsurfaceflinger_hook.so output/

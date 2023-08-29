.DEFAULT_GOAL := package
RELEASE ?= false

.PHONY: clean
clean:
	@cargo clean
	@cd AndroidPtraceInject && \
		make clean

.PHONY: inject
inject:
	@cd AndroidPtraceInject && \
	make RELEASE=$(RELEASE)

.PHONY: hook_lib
hook_lib:
	@echo "Building hook lib…"
ifeq ($(RELEASE), true)
	cargo build --release --target aarch64-linux-android
else
	cargo build --target aarch64-linux-android
endif
	@echo "Built surfaceflinger_hook(lib) successfully"
	
.PHONY: package
package: inject hook_lib
	@rm -rf output/*
	@mkdir -p output

	@cp -rf scripts/* output/

ifeq ($(RELEASE), true)
	@cp -f AndroidPtraceInject/target/aarch64-linux-android/release/inject output/
	@cp -f target/aarch64-linux-android/release/libsurfaceflinger_hook.so
else
	@cp -f AndroidPtraceInject/target/aarch64-linux-android/debug/inject output/
	@cp -f target/aarch64-linux-android/debug/libsurfaceflinger_hook.so output/
endif

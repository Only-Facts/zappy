MODULES = 	\
	cube			\
	console		\
	overlay

all: run

init:
	@mkdir -p modules

$(MODULES): init
	@echo -e "\x1b[1;33m[COMPILING]\x1b[0;90m $@ in WASM..."
	@cargo component build -p $@ --release && \
	cp target/wasm32-wasip1/release/$@.wasm modules/$@.wasm && \
	echo -e "\x1b[1;32m[OK]\x1b[94;3m $@ \x1b[0;90mcompiled & copied to \x1b[4;90mmodules/\x1b[0;90m !\x1b[0m" || \
	echo -e "\x1b[1;31m[ERROR]\x1b[0;90m Failed to compile \x1b[31;3m$@\x1b[0m"

build-modules: $(MODULES)

run: build-modules
	@echo -e "\x1b[1;36m[RUN]\x1b[0;90m Core\x1b[0m"
	@cargo run -p core --release

clean:
	@echo -e "\x1b[1;35m[CLEAN]\x1b[0;90m Cargo & \x1b[4;90mmodules/\x1b[0m"
	@cargo clean
	@rm -rf modules/

re: clean all

.PHONY: all init $(MODULES) build-modules run clean re

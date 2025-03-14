.PHONY: default debug release dev format clippy clean install

PREFIX ?= /usr/local

default: debug

target:
	-btrfs subvolume create target
	mkdir -p target

debug: | target
	cargo build

release: | target
	cargo build --release

dev: format clippy

format: | target
	cargo fmt --verbose

clippy: | target
	cargo clippy

clean:
	rm -rf target
	cargo clean

install: release
	# This rebuilds everything again...
	#DESTDIR= cargo install --path . --no-track --root $(DESTDIR)$(PREFIX)
	install -D target/release/fiv $(DESTDIR)$(PREFIX)/bin/fiv

.PHONY: default debug release dev format clippy clean install

PREFIX ?= /usr/bin/local

default: debug

target:
	-btrfs subvolume create target
	mkdir -p target

debug: | target
	cargo build

release: | target
	cargo build --release

dev: format clippy

format:
	rustfmt --verbose src/main.rs

clippy: | target
	cargo clippy

clean:
	rm -rf target
	cargo clean

install: release
	install -D target/release/fiv $(DESTDIR)$(PREFIX)/fiv

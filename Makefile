.PHONY: dev check build

dev:
	cargo run

check:
	cargo check

build:
	cargo build --release

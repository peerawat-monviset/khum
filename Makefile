.PHONY: dev check build test

dev:
	cargo run

check:
	cargo check

build:
	cargo build --release

test:
	cargo build
	/opt/homebrew/bin/bun test tests/integration.test.js

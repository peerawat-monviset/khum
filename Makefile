.PHONY: dev check build test run

dev:
	cargo run

check:
	cargo check

build:
	cargo build --release

run:
	rm -f promos.txt
	cd frontend && /opt/homebrew/bin/bun run build
	(sleep 1.5 && (open -a Helium "http://localhost:$${PORT:-8000}" 2>/dev/null || open "http://localhost:$${PORT:-8000}")) &
	cargo run

test:
	cargo build
	/opt/homebrew/bin/bun test tests/integration.test.js

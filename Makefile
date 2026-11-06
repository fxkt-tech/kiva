.PHONY: run check fmt lint test wasm

run:
	./scripts/web-dev.sh $(ARGS)

check: fmt lint test wasm

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --locked --all-targets -- -D warnings

test:
	cargo test --locked

wasm:
	cargo check --locked --target wasm32-unknown-unknown

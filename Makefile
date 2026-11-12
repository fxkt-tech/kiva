.PHONY: run check fmt lint test wasm bake-hits

run:
	./scripts/web-dev.sh $(ARGS)

check: fmt lint test wasm

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --locked --all-targets -- -D warnings

test:
	cargo test --locked
	node --test scripts/*-unit.test.cjs

wasm:
	cargo check --locked --target wasm32-unknown-unknown

# Rebuild simulation/combat/hit_tracks.rs from the attack animations. Run after
# changing an attack clip, a clip row, a melee weapon model or an attack duration.
bake-hits:
	cargo test --locked --lib client::hit_bake_tests::bake_hit_tracks -- --ignored --exact --nocapture
	cargo test --locked --lib hit_tracks

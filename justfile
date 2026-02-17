compile:
	cargo build

release:
	cargo build --release

test: check
	cargo test -- --include-ignored

test-ci: check
	cargo test

check:
	cargo fmt
	cargo clippy


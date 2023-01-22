compile:
	cargo build

release:
	cargo build --release

test:
	cargo fmt
	cargo clippy
	cargo test

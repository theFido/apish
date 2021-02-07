doc:
	cargo doc

test:
	cargo test

fmt:
	cargo fmt

clippy:
	cargo clippy

release: doc test fmt clippy
	cargo build --release
export PATH := $(HOME)/.cargo/bin:$(PATH)

.PHONY: setup init test lint check

setup: init
	cargo fetch --locked

init:
	@command -v rustc >/dev/null 2>&1 || { echo 'rustc not found; install the Rust stable toolchain (https://rustup.rs)' >&2; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo 'cargo not found; install the Rust stable toolchain (https://rustup.rs)' >&2; exit 1; }
	@cargo fmt --version >/dev/null 2>&1 || { echo 'rustfmt missing; run rustup component add rustfmt' >&2; exit 1; }
	@cargo clippy --version >/dev/null 2>&1 || { echo 'clippy missing; run rustup component add clippy' >&2; exit 1; }
	@test -f Cargo.toml && test -f Cargo.lock || { echo 'Cargo.toml or Cargo.lock missing; run make from the repository root' >&2; exit 1; }
	@rustc --version
	@cargo --version

test: init
	cargo test --locked --offline

lint: init
	cargo fmt --all -- --check
	cargo clippy --locked --offline --all-targets -- -D warnings

check: lint test
	cargo check --locked --offline

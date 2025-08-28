default: test

fmt:
    cargo fmt --all

lint:
    cargo check
    cargo clippy --all-features

build: fmt lint
    cargo build

test: build
    cargo test

run:
    cargo run --release

bench:
    cargo bench

all: test bench run

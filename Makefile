BINARY_NAME=r-heatmap

all: build

build:
	cargo build --release

run:
	cargo run

clean:
	cargo clean
	rm -f r-heatmap.log

test:
	cargo test

check:
	cargo check

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

install:
	cargo install --path .

.PHONY: all build run clean test check lint fmt install

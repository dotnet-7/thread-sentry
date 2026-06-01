.PHONY: all build test run clean help install examples benchmark doc

all: build test

build:
	@echo "Building Thread-Sentry..."
	cargo build --release

test:
	@echo "Running tests..."
	cargo test --release

run: build
	@echo "Running demo..."
	cargo run --release --example demo

examples: build
	@echo "Running all examples..."
	@echo "=== Demo ==="
	cargo run --release --example demo
	@echo ""
	@echo "=== Benchmark ==="
	cargo run --release --example benchmark
	@echo ""
	@echo "=== Real World ==="
	cargo run --release --example real_world
	@echo ""
	@echo "=== Advanced Usage ==="
	cargo run --release --example advanced_usage

benchmark: build
	@echo "Running performance benchmark..."
	cargo run --release --example benchmark

doc:
	@echo "Generating documentation..."
	cargo doc --release --open

clean:
	@echo "Cleaning build artifacts..."
	cargo clean

install: build
	@echo "Installing Thread-Sentry..."
	cargo install --path .

help:
	@echo "Thread-Sentry Makefile Commands:"
	@echo "  make build      - Build the project"
	@echo "  make test       - Run unit tests"
	@echo "  make run        - Run the demo example"
	@echo "  make examples   - Run all examples"
	@echo "  make benchmark  - Run performance benchmark"
	@echo "  make doc        - Generate and open documentation"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make install    - Install the library"
	@echo "  make help       - Show this help message"

.DEFAULT_GOAL := help
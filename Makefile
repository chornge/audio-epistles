# Common development tasks

.PHONY: all build release test check clippy fmt clean run doc audit help

# Default target
all: check test build

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run all tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Type check without building
check:
	cargo check

# Run clippy linter
clippy:
	cargo clippy -- -D warnings

# Format code
fmt:
	cargo fmt

# Check formatting without modifying
fmt-check:
	cargo fmt -- --check

# Clean build artifacts
clean:
	cargo clean
	rm -f assets/*.mp3 assets/*.mp4

# Run the application
run:
	cargo run --release

# Run with debug logging
run-debug:
	RUST_LOG=debug cargo run

# Run with trace logging
run-trace:
	RUST_LOG=trace cargo run

# Generate documentation
doc:
	cargo doc --no-deps --open

# Security audit (requires cargo-audit)
audit:
	cargo audit

# Install development dependencies
dev-setup:
	cargo install cargo-audit cargo-watch
	@echo "Ensure chromedriver, ffmpeg, and yt-dlp are installed"
	@echo "  macOS: brew install chromedriver ffmpeg yt-dlp"

# Watch for changes and run tests
watch:
	cargo watch -x test

# Watch for changes and check
watch-check:
	cargo watch -x check

# Docker build
docker-build:
	docker build -t audio_epistles .

# Docker run
docker-run:
	docker-compose up -d

# Docker stop
docker-stop:
	docker-compose down

# Docker logs
docker-logs:
	docker-compose logs -f

# Full CI check (what CI runs)
ci: fmt-check clippy test
	@echo "CI checks passed!"

# Pre-commit checks
pre-commit: fmt clippy test
	@echo "Pre-commit checks passed!"

# Help
help:
	@echo "Audio Epistles - Available targets:"
	@echo ""
	@echo "  Build:"
	@echo "    build        - Build debug version"
	@echo "    release      - Build release version"
	@echo "    clean        - Clean build artifacts and media files"
	@echo ""
	@echo "  Test & Lint:"
	@echo "    test         - Run all tests"
	@echo "    test-verbose - Run tests with output"
	@echo "    check        - Type check without building"
	@echo "    clippy       - Run clippy linter"
	@echo "    fmt          - Format code"
	@echo "    fmt-check    - Check formatting"
	@echo "    audit        - Run security audit"
	@echo ""
	@echo "  Run:"
	@echo "    run          - Run application (release)"
	@echo "    run-debug    - Run with debug logging"
	@echo "    run-trace    - Run with trace logging"
	@echo ""
	@echo "  Development:"
	@echo "    dev-setup    - Install dev dependencies"
	@echo "    watch        - Watch and run tests"
	@echo "    watch-check  - Watch and type check"
	@echo "    doc          - Generate and open docs"
	@echo "    pre-commit   - Run pre-commit checks"
	@echo "    ci           - Run full CI checks"
	@echo ""
	@echo "  Docker:"
	@echo "    docker-build - Build Docker image"
	@echo "    docker-run   - Start with docker-compose"
	@echo "    docker-stop  - Stop docker-compose"
	@echo "    docker-logs  - View docker logs"

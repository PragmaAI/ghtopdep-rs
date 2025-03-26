.PHONY: build test clean install release help

# Default target
all: build

# Variables
CARGO := cargo
BINARY_NAME := ghtopdep-rs
INSTALL_DIR := $(HOME)/.local/bin
RELEASE_FLAGS := --release
TEST_REPO := near/near-sdk-rs

# Help command
help:
	@echo "Available commands:"
	@echo "  make build      - Build the project in debug mode"
	@echo "  make release    - Build the project in release mode"
	@echo "  make test       - Run tests"
	@echo "  make run        - Run with test repository"
	@echo "  make install    - Install to $(INSTALL_DIR)"
	@echo "  make uninstall  - Remove from $(INSTALL_DIR)"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make help       - Show this help message"

# Build in debug mode
build:
	$(CARGO) build

# Build in release mode
release:
	$(CARGO) build $(RELEASE_FLAGS)

# Run tests
test:
	$(CARGO) test

# Run tests with output
test-verbose:
	$(CARGO) test -- --nocapture

# Run ignored tests (like integration tests that make real network requests)
test-all:
	$(CARGO) test -- --include-ignored

# Run with a test repository
run: build
	$(CARGO) run -- $(TEST_REPO)

# Run with a test repository in release mode
run-release: release
	./target/release/$(BINARY_NAME) $(TEST_REPO)

# Install the binary
install: release
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@echo "Installed $(BINARY_NAME) to $(INSTALL_DIR)"
	@if [[ ":$(PATH):" != *":$(INSTALL_DIR):"* ]]; then \
		echo "Warning: $(INSTALL_DIR) is not in your PATH"; \
		echo "Consider adding 'export PATH=\$$PATH:$(INSTALL_DIR)' to your shell configuration"; \
	fi

# Uninstall the binary
uninstall:
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Removed $(BINARY_NAME) from $(INSTALL_DIR)"

# Clean build artifacts
clean:
	$(CARGO) clean 
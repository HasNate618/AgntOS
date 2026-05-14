.PHONY: build check run inspect clean test

# Build all crates (release)
build:
	cargo build --release

# Quick check (no artifacts)
check:
	cargo check

# Run agntctl with args (e.g. make ARGS="inspect")
run:
	cargo run --bin agntctl -- $(ARGS)

# Quick system inspection
inspect:
	cargo run --bin agntctl -- inspect

# Run agntd daemon
agent:
	cargo run --bin agntd

# Clean build artifacts
clean:
	cargo clean

# Watch for changes (requires cargo-watch)
watch:
	cargo watch -x check

# Install into dev VM profile
install:
	sudo cp target/release/agntctl /usr/local/bin/
	sudo cp target/release/agntd /usr/local/bin/

# Rebuild VM NixOS config (when using flake)
vm-rebuild:
	sudo nixos-rebuild switch --flake .

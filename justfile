# List the available recipes
default:
    @just --list

# Compile the launcher
build:
    cargo build

# Format the code
fmt:
    cargo fmt

# Run the tests
test:
    cargo test

# Lint, treating warnings as errors, and check formatting
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings


pre-commit: lint test

install-pre-commit:
    @echo "Installing pre-commit hook..."
    @mkdir -p .git/hooks
    @echo '#!/bin/sh' > .git/hooks/pre-commit
    @echo 'just pre-commit' >> .git/hooks/pre-commit
    @echo 'exit $?' >> .git/hooks/pre-commit
    @chmod +x .git/hooks/pre-commit
    @echo "✅ Pre-commit hook installed successfully!"
    @echo "The hook will run 'just pre-commit' before each commit."

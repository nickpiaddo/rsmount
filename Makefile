FIGURES =$(wildcard assets/diagrams/namespaces/d2/*.d2)
SVG_OUTPUT_DIR = assets/diagrams/svg

# Build the library
all:
	cargo build

# Build figures and diagrams
build-figs: $(FIGURES)
	./scripts/build-diagrams --output $(SVG_OUTPUT_DIR) $?

# Build the library documentation
doc:
	cargo doc --no-deps -p rsmount-sys -p rsmount

# Rebuild documentation and diagrams
doc-rebuild: build-figs doc

# Publish crate to crates.io
do-publish: build-figs
    cargo publish

# Dry run cargo publish
publish: test-all doc-rebuild
    cargo publish --dry-run

# Run unit/integration tests
test:
	cargo nextest run

# Run doc tests
test-doc:
	cargo test --doc

# Run all tests
test-all: test test-doc

FIGURES =$(wildcard assets/diagrams/namespaces/d2/*.d2)
SVG_OUTPUT_DIR = assets/diagrams/svg

# Build the library
all:
	cargo build

# Build figures and diagrams
fig-build: $(FIGURES)
	./scripts/build-diagrams --output $(SVG_OUTPUT_DIR) $?

# Build the library documentation
doc:
	cargo doc --no-deps -p rsmount-sys -p rsmount

# Rebuild documentation and diagrams
doc-rebuild: fig-build doc

# Run unit/integration tests
test:
	cargo nextest run

# Run doc tests
doctest:
	cargo test --doc

# Run all tests
fulltest: test doctest

alias d := doc
alias dr := doc-rebuild
alias t := test
alias tu := test-unit
alias td := test-doc
alias tdu := test-doc-unit
alias ta := test-all

packages := "-p rsmount-sys -p rsmount"
dependencies := ""
figures := "assets/diagrams/namespaces/d2/*.d2"
svg-output-dir := "assets/diagrams/svg"

# Build the library
default:
	cargo build {{packages}}

# Build figures and diagrams
build-figs:
 ./scripts/build-diagrams --output {{svg-output-dir}} {{figures}}

# Build the library documentation
doc:
	cargo doc --no-deps {{packages}} {{dependencies}}

# Build diagrams and library documentation
doc-rebuild: build-figs doc

# Publish crate to crates.io
do-publish: build-figs
 cargo publish

# Dry run cargo publish
publish: test-all doc-rebuild
  cargo publish --dry-run

# Run all unit/integration tests
test:
	cargo nextest run {{packages}}

# Run unit test named TESTNAME
test-unit TESTNAME:
	cargo nextest run {{TESTNAME}} {{packages}}

# Run doc tests
test-doc:
	cargo test --doc {{packages}}

# Run doc tests containing the string [TESTNAME]
test-doc-unit TESTNAME:
    cargo test --doc {{TESTNAME}} {{packages}}

# Run unit, integration, and doc tests
test-all: test test-doc

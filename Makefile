# Build the library
all:
	cargo build

# Build the library documentation
doc:
	cargo doc --no-deps -p rsmount-sys -p rsmount

# Run unit/integration tests
test:
	cargo nextest run

# Run doc tests
doctest:
	cargo test --doc

# Run all tests
fulltest: test doctest

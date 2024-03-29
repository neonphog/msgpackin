# Msgpackin Makefile

.PHONY: all bump_core bump publish_core publish test static tools tool_rust tool_fmt tool_readme

SHELL = /usr/bin/env sh -eu

all: test

bump_core:
	@if [ "$(ver)x" = "x" ]; then \
		echo "# USAGE: 'make bump_core ver=0.0.1-alpha.42'"; \
		exit 1; \
	fi
	sed -i'' 's/^version = \"[^"]*"$$/version = "$(ver)"/g' crates/msgpackin_core/Cargo.toml; \
	sed -i'' 's/^msgpackin_core = { version = \"[^"]*"/msgpackin_core = { version = "$(ver)"/g' crates/msgpackin/Cargo.toml; \

publish_core: test
	git diff --exit-code
	cargo publish --manifest-path crates/msgpackin_core/Cargo.toml
	VER="msgpackin_core-v$$(grep version crates/msgpackin_core/Cargo.toml | head -1 | cut -d ' ' -f 3 | cut -d \" -f 2)"; git tag -a $$VER -m $$VER
	git push --tags

publish: test
	git diff --exit-code
	cargo publish --manifest-path crates/msgpackin/Cargo.toml
	VER="msgpackin-v$$(grep version crates/msgpackin/Cargo.toml | head -1 | cut -d ' ' -f 3 | cut -d \" -f 2)"; git tag -a $$VER -m $$VER
	git push --tags

test: static tools
	RUST_BACKTRACE=1 ./features-test.bash
	@if [ "${CI}x" != "x" ]; then git diff --exit-code; fi

static: tools
	cargo fmt -- --check
	cargo clippy --all-features
	cargo readme -r crates/msgpackin_core -o README.md
	cargo readme -r crates/msgpackin -o README.md
	cargo readme -r crates/msgpackin -o ../../README.md

tools: tool_rust tool_fmt tool_clippy tool_readme

tool_rust:
	@if rustup --version >/dev/null 2>&1; then \
		echo "# Makefile # found rustup, setting override stable"; \
		rustup override set stable; \
	else \
		echo "# Makefile # rustup not found, hopefully we're on stable"; \
	fi;

tool_fmt: tool_rust
	@if ! (cargo fmt --version); \
	then \
		if rustup --version >/dev/null 2>&1; then \
			echo "# Makefile # installing rustfmt with rustup"; \
			rustup component add rustfmt; \
		else \
			echo "# Makefile # rustup not found, cannot install rustfmt"; \
			exit 1; \
		fi; \
	else \
		echo "# Makefile # rustfmt ok"; \
	fi;

tool_clippy: tool_rust
	@if ! (cargo clippy --version); \
	then \
		if rustup --version >/dev/null 2>&1; then \
			echo "# Makefile # installing clippy with rustup"; \
			rustup component add clippy; \
		else \
			echo "# Makefile # rustup not found, cannot install clippy"; \
			exit 1; \
		fi; \
	else \
		echo "# Makefile # clippy ok"; \
	fi;

tool_readme: tool_rust
	@if ! (cargo readme --version); \
	then \
		cargo install cargo-readme; \
	else \
		echo "# Makefile # readme ok"; \
	fi;

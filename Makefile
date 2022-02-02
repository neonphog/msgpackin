# Lair Makefile

.PHONY: all bump publish test tools tool_rust tool_fmt tool_readme

#RUSTFLAGS += ...

SHELL = /usr/bin/env sh

ENV = RUSTFLAGS='$(RUSTFLAGS)' CARGO_BUILD_JOBS='$(shell nproc || sysctl -n hw.physicalcpu)' NUM_JOBS='$(shell nproc || sysctl -n hw.physicalcpu)' CARGO_TARGET_DIR='$(shell pwd)/target'

all: test

bump:
	@if [ "$(ver)x" = "x" ]; then \
		echo "# USAGE: 'make bump ver=0.0.1-alpha.42'"; \
		exit 1; \
	fi
	@for toml in $$(find crates -name Cargo.toml); do \
		echo "# updating version in $${toml} to $(ver)"; \
		sed -i'' 's/^version = \"[^"]*"$$/version = "$(ver)"/g' $${toml}; \
		sed -i'' 's/^msgpackin_core = { version = \"[^"]*"/msgpackin_core = { version = "=$(ver)"/g' $${toml}; \
	done

publish: tools
	git diff --exit-code
	cargo publish --manifest-path crates/msgpackin_core/Cargo.toml
	#echo "-- wait for crates.io... --"; sleep 30
	#cargo publish --manifest-path crates/msgpackin/Cargo.toml
	VER="v$$(grep version crates/msgpackin_core/Cargo.toml | head -1 | cut -d ' ' -f 3 | cut -d \" -f 2)"; echo git tag -a $$VER -m $$VER
	echo git push --tags

test: tools
	$(ENV) cargo fmt -- --check
	$(ENV) cargo clippy
	$(ENV) RUST_BACKTRACE=1 cargo test --all-targets --no-run
	$(ENV) RUST_BACKTRACE=1 cargo test
	$(ENV) cargo readme -r crates/msgpackin_core -o README.md
	#$(ENV) cargo readme -r crates/msgpackin -o README.md
	#$(ENV) cargo readme -r crates/msgpackin -o ../../README.md
	@if [ "${CI}x" != "x" ]; then git diff --exit-code; fi

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

#
# Test and build elemental-chat DNA Project
#
# This Makefile is primarily instructional; you can simply enter the Nix environment for
# holochain development (supplied by holo-nixpkgs; see nixpkgs.nix) via `nix-shell` and run
# `make test` directly, or build a target directly eg. `nix-build -A elemental-chat`.
#
SHELL		= bash
DNANAME		= elemental-chat
DNA		= $(DNANAME).dna.gz
WASM		= target/wasm32-unknown-unknown/release/chat.wasm

# External targets; Uses a nix-shell environment to obtain Holochain runtimes, run tests, etc.
.PHONY: all FORCE
all: nix-test

# nix-test, nix-install, ...
nix-%:
	nix-shell --pure --run "make $*"

# Internal targets; require a Nix environment in order to be deterministic.
# - Uses the version of `dna-util`, `holochain` on the system PATH.
# - Normally called from within a Nix environment, eg. run `nix-shell`
.PHONY:		rebuild install build
rebuild:	clean build

install:	build

build:		$(DNA)

# Package the DNA from the built target release WASM
$(DNA):		$(WASM) FORCE
	@echo "Packaging DNA:"
	@dna-util -c $(DNANAME).dna.workdir
	@ls -l $@

# Recompile the target release WASM
$(WASM): FORCE
	@echo "Building  DNA WASM:"
	@RUST_BACKTRACE=1 CARGO_TARGET_DIR=target cargo build \
	    --release --target wasm32-unknown-unknown

.PHONY: test test-all test-unit test-e2e
test-all:	test

test:		test-unit test-e2e # test-stress # re-enable when Stress tests end reliably

test-unit:
	RUST_BACKTRACE=1 cargo test \
	    -- --nocapture

# test-dna, test-dna-standard ==> npm run test:standard
test-dna:	$(DNA) FORCE
	@echo "Starting Scenario tests in $$(pwd)..."
	cd tests && ( [ -d  node_modules ] || npm install ) && npm run test

test-dna-%:	$(DNA) FORCE
	@echo "Starting '$*' Scenario tests in $$(pwd)..."
	cd tests && ( [ -d  node_modules ] || npm install ) && npm run test:$*

test-e2e:	test-dna


# Generic targets; does not require a Nix environment
.PHONY: clean
clean:
	rm -rf \
	   tests/node_modules \
	   .cargo \
	   target \
	   $(DNA)

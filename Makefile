#
# Test and build Elemental Chat Project
#
# This Makefile is primarily instructional; you can simply enter the Nix environment for
# holochain-rust development (supplied by holonix;) via `nix-shell` and run
# `make test` directly, or build a target directly, eg. `nix-build -A elemental-chat
#
SHELL		= bash
DNANAME		= elemental-chat
DNA			= $(DNANAME).dna
HAPP		= $(DNANAME).happ
WASM		= target/wasm32-unknown-unknown/release/chat.wasm
CHAT_INTEGRITY_WASM	= target/wasm32-unknown-unknown/release/chat_integrity.wasm
PROFILE_WASM = target/wasm32-unknown-unknown/release/profile.wasm
PROFILE_INTEGRITY_WASM = target/wasm32-unknown-unknown/release/profile_integrity.wasm
MEM_MANAGER_WASM = target/wasm32-unknown-unknown/release/membrane_manager_integrity.wasm

.PHONY: DNAs

dnas:
	mkdir -p ./dnas
dnas/joining-code-factory.dna:	dnas
	curl 'https://holo-host.github.io/joining-code-happ/releases/downloads/0_3_0/joining-code-factory.0_3_0.dna' -o $@

DNAs: dnas/joining-code-factory.dna

# External targets; Uses a nix-shell environment to obtain Holochain runtimes, run tests, etc.
.PHONY: all FORCE
all: nix-test

# nix-test, nix-install, ...
nix-%:
	nix-shell --pure --run "make $*"

# Internal targets; require a Nix environment in order to be deterministic.
# - Uses the version of `hc` and `holochain` on the system PATH.
# - Normally called from within a Nix environment, eg. run `nix-shell`
.PHONY: rebuild install build build-cargo build-dna
rebuild: clean build

install: build

build: build-cargo build-dna

build: $(DNA)

# Package the DNA from the built target release WASM
$(DNA):	 $(WASM) FORCE
	@echo "Packaging DNA: hc path: `which hc`"
	@hc dna pack . -o $(DNA)
	@hc app pack . -o $(HAPP)
	@ls -l $@

# Recompile the target release WASM
$(WASM): FORCE
	@echo "Building  DNA WASM:"
	@RUST_BACKTRACE=1 CARGO_TARGET_DIR=target cargo build \
	    --release --target wasm32-unknown-unknown
	@echo "Optimizing wasms:"
	@wasm-opt -Oz $(WASM) --output $(WASM)
	@wasm-opt -Oz $(CHAT_INTEGRITY_WASM) --output $(CHAT_INTEGRITY_WASM)
	@wasm-opt -Oz $(PROFILE_WASM) --output $(PROFILE_WASM)
	@wasm-opt -Oz $(PROFILE_INTEGRITY_WASM) --output $(PROFILE_INTEGRITY_WASM)
	@wasm-opt -Oz $(MEM_MANAGER_WASM) --output $(MEM_MANAGER_WASM)

.PHONY: test test-all test-unit test-e2e test-dna test-dna-debug test-stress test-sim2h test-node
test-all: test

test: test-unit test-e2e # test-stress # re-enable when Stress tests end reliably

test-unit: $(DNA) FORCE
	RUST_BACKTRACE=1 cargo test \
	    -- --nocapture

test-dna: DNAs $(DNA) FORCE
	@echo "Starting Scenario tests in $$(pwd)..."; \
	    cd tests && ( [ -d  node_modules ] || npm install ) && npm test

test-dna-debug: DNAs $(DNA) FORCE
	@echo "Starting Scenario tests in $$(pwd)..."; \
	    cd tests && ( [ -d node_modules ] || npm install ) && npm run test:debug

test-behavior: DNAs $(DNA) FORCE
	@echo "Starting Scenario tests in $$(pwd)..."; \
	    cd tests && ( [ -d  node_modules ] || npm install ) && npm run test:behavior

test-e2e: test-dna

#############################
# █▀█ █▀▀ █░░ █▀▀ ▄▀█ █▀ █▀▀
# █▀▄ ██▄ █▄▄ ██▄ █▀█ ▄█ ██▄
#############################
# requirements
# - cargo-edit crate: `cargo install cargo-edit`
# - jq linux terminal tool : `sudo apt-get install jq`
# How to make a release?
# make update

update:
	echo '⚙️  Updating hdk crate...'
	cargo upgrade hdk@=$(shell jq .hdk ./version-manager.json) --workspace
	echo '⚙️  Updating holo_hash crate...'
	cargo upgrade holo_hash@=$(shell jq .holo_hash ./version-manager.json) --workspace
	echo '⚙️  Updating holochain crate...'
	cargo upgrade holochain@=$(shell jq .holochain ./version-manager.json) --workspace
	echo '⚙️  Updating hc_utils crate...'
	cargo upgrade hc_utils@=$(shell jq .hc_utils ./version-manager.json) --workspace	
	echo '⚙️  Updating holochainVersionId in nix...'
	sed -i -e 's/^  holonixRevision = .*/  holonixRevision = $(shell jq .holonix_rev ./version-manager.json);/' config.nix;\
	sed -i -e 's/^  holochainVersionId = .*/  holochainVersionId = $(shell jq .holochain_rev ./version-manager.json);/' config.nix;\
	echo '⚙️  Building dnas and happ...'
	rm -rf Cargo.lock
	make nix-build
	echo '⚙️  Running tests...'
	make nix-test-dna-debug
	
# release-%:
# 	echo '⚙️  Making new release...'
# 	make HC_REV=$(HC_REV) github-release-$*
# 	echo '🚀  Successful release elemental-chat '$*


# Generic targets; does not require a Nix environment
.PHONY: clean
clean:
	rm -rf \
	    tests/node_modules \
	    .cargo \
	    target \
	    dnas

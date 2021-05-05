#
# Test and build Elemental Chat Project
#
# This Makefile is primarily instructional; you can simply enter the Nix environment for
# holochain-rust development (supplied by holonix;) via `nix-shell` and run
# `make test` directly, or build a target directly, eg. `nix-build -A elemental-chat
#
SHELL		= bash
DNANAME		= elemental-chat
DNA		= $(DNANAME).dna
HAPP		= $(DNANAME).happ
WASM		= target/wasm32-unknown-unknown/release/chat.wasm

# External targets; Uses a nix-shell environment to obtain Holochain runtimes, run tests, etc.
.PHONY: all FORCE
all: nix-test

# nix-test, nix-install, ...
nix-%:
	nix-shell --pure --run "make $*"

# Internal targets; require a Nix environment in order to be deterministic.
# - Uses the version of `hc` and `holochain` on the system PATH.
# - Normally called from within a Nix environment, eg. run `nix-shell`
.PHONY:		rebuild install build build-cargo build-dna
rebuild:	clean build

install:	build

build:	build-cargo build-dna

build:		$(DNA)

# Package the DNA from the built target release WASM
$(DNA):		$(WASM) FORCE
	@echo "Packaging DNA:"
	@hc dna pack . -o $(DNA)
	@hc app pack . -o $(HAPP)
	@ls -l $@

# Recompile the target release WASM
$(WASM): FORCE
	@echo "Building  DNA WASM:"
	@RUST_BACKTRACE=1 CARGO_TARGET_DIR=target cargo build \
	    --release --target wasm32-unknown-unknown

.PHONY: test test-all test-unit test-e2e test-dna test-dna-debug test-stress test-sim2h test-node
test-all:	test

test:		test-unit test-e2e # test-stress # re-enable when Stress tests end reliably

test-unit:
	RUST_BACKTRACE=1 cargo test \
	    -- --nocapture

test-dna:	$(DNA) FORCE
	@echo "Starting Scenario tests in $$(pwd)..."; \
	    cd tests && ( [ -d  node_modules ] || npm install ) && npm test

test-dna-debug:
	@echo "Starting Scenario tests in $$(pwd)..."; \
	    cd tests && ( [ -d  node_modules ] || npm install ) && npm run test:standard

test-behavior:
	@echo "Starting Scenario tests in $$(pwd)..."; \
	    cd tests && ( [ -d  node_modules ] || npm install ) && npm run test:behavior

test-e2e:	test-dna

#############################
# █▀█ █▀▀ █░░ █▀▀ ▄▀█ █▀ █▀▀
# █▀▄ ██▄ █▄▄ ██▄ █▀█ ▄█ ██▄
#############################
# How to make a release?
# make HC_REV="HC_REV" release-0.0.0-alpha0

update-release-%:
	cd zomes/chat/ && sed -i -e 's/^version = .*/version = "$*"/' Cargo.toml

update-hc:
	make HC_REV=$(HC_REV) update-hc-sha
	make HC_REV=$(HC_REV) update-nix-by-failure
	make HC_REV=$(HC_REV) update-hc-cargoSha

update-hc-sha:
	@if [ $(HC_REV) ]; then\
		echo "⚙️  Updating elemental-chat using holochain rev: $(HC_REV)";\
		echo "✔  Updating hdk rev in Cargo.toml...";\
		sed -i -e 's/^hdk = .*/hdk = {git ="https:\/\/github.com\/holochain\/holochain", rev = "$(HC_REV)", package = "hdk"}/' Cargo.toml;\
		echo "✔  Replacing rev...";\
		sed -i -e 's/^     rev = .*/     rev = "$(HC_REV)";/' default.nix;\
		echo "✔  Replacing sha256...";\
		sed -i -e 's/^     sha256 = .*/     sha256 = "$(shell nix-prefetch-url --unpack "https://github.com/holochain/holochain/archive/$(HC_REV).tar.gz")";/' default.nix;\
	else \
		echo "No holochain rev provided"; \
  fi

update-nix-by-failure:
	@if [ $(HC_REV) ]; then\
		echo "➳  Corrupting cargoSha256...";\
		sed -i -e 's/^     cargoSha256 = .*/     cargoSha256 = "000000000000000000000000000000000000000000000000000a";/' default.nix;\
		echo "➳  Getting cargoSha256... This can take a while...";\
		nix-shell &>nix.log || echo "This was ment to fail :)...";\
	else \
		echo "No holochain rev provided"; \
  fi


update-hc-cargoSha:
	@if [ $(HC_REV) ]; then\
		echo "➳  Waiting for 5s..."$*;\
		sleep 5;\
		echo "✔  Replacing cargoSha256...";\
		$(eval CARGOSHA256=$(shell sh -c "grep "got" ./nix.log" | awk '{print $$2}'))\
		sed -i -e 's/^     cargoSha256 = .*/     cargoSha256 = "$(CARGOSHA256)";/' default.nix;\
	else \
		echo "No holochain rev provided"; \
  fi

github-release-%:
	@echo "TODO";\
	echo "Creating github-release for version $*"
	cp elemental-chat.happ elemental-chat.$(shell echo $* | tr .- _).happ
	cp elemental-chat.dna elemental-chat.$(shell echo $* | tr .- _).dna
	sh ./gh-release.sh $* "holochain rev: $(HC_REV)"

release-%:
	echo '⚙️  Editing necessary files required for update...'
	make update-release-$*
	make HC_REV=$(HC_REV) update-hc
	echo '⚙️  Building dnas and happ...'
	rm -rf Cargo.lock
	make nix-build
	echo '⚙️  Running tests...'
	make nix-test-dna-debug
	echo '⚙️  Commit updates to current branch...'
	git checkout -b release-$*
	git add zomes/ Cargo.toml default.nix
	git commit -m v$*
	git push origin HEAD
	echo '⚙️  Making new release...'
	make HC_REV=$(HC_REV) github-release-$*
	echo '🚀  Successful release elemental-chat '$*


# Generic targets; does not require a Nix environment
.PHONY: clean
clean:
	rm -rf \
	    tests/node_modules \
	    .cargo \
	    target \
	    $(DNA)

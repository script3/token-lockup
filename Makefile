default: build

test: build
	cargo test --all --tests

build:
	mkdir -p target/wasm32-unknown-unknown/optimized

	cargo rustc --manifest-path=Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/token_lockup.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/token_lockup.wasm

	cd target/wasm32-unknown-unknown/optimized/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

fmt:
	cargo fmt --all

clean:
	cargo clean


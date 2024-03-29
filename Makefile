default: build

test: build
	cargo test --all --tests

build:
	mkdir -p target/wasm32-unknown-unknown/optimized

	cargo rustc --manifest-path=token-lockup/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features standard
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/token_lockup.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/standard_token_lockup.wasm

	cargo rustc --manifest-path=token-lockup/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features blend --no-default-features 
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/token_lockup.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/blend_token_lockup.wasm

	cd target/wasm32-unknown-unknown/optimized/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

fmt:
	cargo fmt --all

clean:
	cargo clean


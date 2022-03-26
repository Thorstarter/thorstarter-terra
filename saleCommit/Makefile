test:; RUST_BACKTRACE=1 cargo test --lib
schema:; cargo run --example schema
build:; cargo build --release --target wasm32-unknown-unknown
optimize:; docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.4
.PHONY: schema

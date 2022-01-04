## Thorstarter Terra Contracts

_Thorstarter's contracts for the Terra blockchain_

### Developing

For each contract folder, you can run the following:

```sh
# Run tests
make
# Re-build schema/*.json files
make schema
# Build contract into wasm bundle
make build
# Optimize wasm bytecode for production
make optimize
```

_Make sure to install the wasm target if needed `rustup target add wasm32-unknown-unknown`_

### Deploying

Using scripts

```
node scripts/deployTiers.js
```

Using the CLI

```
terrad tx wasm store tiers/artifacts/thorstarter_terra_tiers.wasm --from test1 --chain-id=localterra --gas=auto --fees=100000uluna --broadcast-mode=block
terrad tx wasm instantiate 1 '{"token":...}' --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block
terrad tx wasm execute terra18vd8fpwxzck93qlwghaj6arh4p7c5n896xzem5 '{"increment":{}}' --from test1 --chain-id=localterra --gas=auto --fees=1000000uluna --broadcast-mode=block
terrad query wasm contract-store terra18vd8fpwxzck93qlwghaj6arh4p7c5n896xzem5 '{"get_count":{}}'
```

### License

MIT

network 				?= devnet  # network := devnet|mainnet|testnet
sender 					?= juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
build_dir 				?= ./builds
wasm_filename 			?= cw_house_staking.wasm
cw20					?= juno12v2yh574zf3t2wuaaec45trypxfgst3zf9hdm57zhzvcu9a0x3ts27qcah
acl						?= juno1p6acvv7mcqa57la3pk0m4ep854jpfpufcryyy5tga899g789y95qh9z3v2

# build optimized WASM artifact
build:
	./bin/build

# deploy WASM file (generated from `make build`)
deploy:
	./bin/deploy ./artifacts/$(wasm_filename) $(network) $(sender) $(tag)

# instantiate last contract to be deployed using code ID in release dir code-id file
instantiate:
	./bin/instantiate $(network) $(sender) $(tag) $(cw20) $(label)

instantiate-with-acl:
	./bin/instantiate $(network) $(sender) $(tag) $(cw20) $(acl) $(label)

# run all unit tests
test:
	RUST_BACKTRACE=1 cargo unit-test

# Generate the contract's JSONSchema JSON files in schemas/
schemas:
	cargo schema

# Run/start local "devnet" validator docker image	
devnet:
	./bin/devnet

delegate:
	./client.sh delegate $(network) $(tag) $(sender)

select:
	./client.sh query-select $(network) $(tag)

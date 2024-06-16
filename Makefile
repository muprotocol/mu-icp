build-mu_smart_contract:
	CANISTER_CANDID_PATH_EXCHANGE_RATE_CANISTER="$(shell pwd)/nns-modules/xrc.did" \
	CANISTER_ID_EXCHANGE_RATE_CANISTER=uf6dk-hyaaa-aaaaq-qaaaq-cai \
	cargo build --target wasm32-unknown-unknown --profile canister-release --package mu_smart_contract
	#candid-extractor ${TARGET_DIR}/mu_smart_contract.wasm > src/mu_smart_contract/mu_smart_contract.did

deploy-all: create-canisters deploy-exchange_rate_canister deploy-mu_smart_contract

run-e2e-tests:
	POCKET_IC_BIN=~/.local/share/dfx/bin/pocket-ic \
	CANISTER_CANDID_PATH_MU_SMART_CONTRACT="$(shell pwd)/src/mu_smart_contract/mu_smart_contract.did" \
	CANISTER_ID_MU_SMART_CONTRACT=bd3sg-teaaa-aaaaa-qaaba-cai \
	CANISTER_CANDID_PATH_LEDGER_CANISTER="$(shell pwd)/nns-modules/ledger.did" \
	CANISTER_ID_LEDGER_CANISTER=ryjl3-tyaaa-aaaaa-aaaba-cai \
	cargo test --package e2e-tests

test: build-mu_smart_contract run-e2e-tests

clean:
	rm -rf .dfx
	rm -rf dist
	rm -rf node_modules
	rm -rf src/declarations
	rm -rf src/mu_smart_contract/src/declarations/
	rm -rf e2e-tests/src/declarations
	rm -f .env

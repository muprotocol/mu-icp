ifdef DFX_NETWORK
	NETWORK := $(DFX_NETWORK)
else
	NETWORK := local
endif

ifdef CARGO_TARGET_DIR
	TARGET_DIR := ${CARGO_TARGET_DIR}/wasm32-unknown-unknown/canister-release
else
	TARGET_DIR := $(pwd)/target/wasm32-unknown-unknown/canister-release/mu_smart_contract.wasm
endif

ifeq (${NETWORK}, ic)
	KEY := default
else
	KEY := default
endif

create-canisters:
	dfx canister create --all --network "${NETWORK}"

deploy-exchange_rate_canister:
	dfx deploy exchange_rate_canister

deploy-mu_smart_contract:
	dfx generate
	CANISTER_CANDID_PATH_EXCHANGE_RATE_CANISTER=../../nns-modules/xrc.did \
	dfx deploy mu_smart_contract --network "${NETWORK}"

build-mu_smart_contract:
	CANISTER_CANDID_PATH_EXCHANGE_RATE_CANISTER=../../nns-modules/xrc.did \
	cargo build --target wasm32-unknown-unknown --profile canister-release --package mu_smart_contract
	#candid-extractor ${TARGET_DIR}/mu_smart_contract.wasm > src/mu_smart_contract/mu_smart_contract.did

deploy-all: create-canisters deploy-exchange_rate_canister deploy-mu_smart_contract

run-e2e-tests:
	POCKET_IC_BIN=~/.local/share/dfx/bin/pocket-ic \
	CANISTER_CANDID_PATH_MU_SMART_CONTRACT="$(shell pwd)/src/mu_smart_contract/mu_smart_contract.did" \
	CANISTER_ID_MU_SMART_CONTRACT=bd3sg-teaaa-aaaaa-qaaba-cai \
	CANISTER_CANDID_PATH_LEDGER_CANISTER="$(shell pwd)/nns-modules/ledger.did" \
	CANISTER_ID_LEDGER_CANISTER=ryjl3-tyaaa-aaaaa-aaaba-cai \
	cargo --config CARGO_MANIFEST_DIR=\"e2e-tests/\" test

test: build-mu_smart_contract run-e2e-tests

clean:
	rm -rf .dfx
	rm -rf dist
	rm -rf node_modules
	rm -rf src/declarations
	rm -f .env

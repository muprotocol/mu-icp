ifdef DFX_NETWORK
	NETWORK := $(DFX_NETWORK)
else
	NETWORK := local
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
	CANISTER_CANDID_PATH_EXCHANGE_RATE_CANISTER=../declarations/exchange-rate-canister/exchange-rate-canister.did \
	candid-extractor ~/.cargo-target/wasm32-unknown-unknown/release/mu_smart_contract.wasm > src/mu_smart_contract/mu_smart_contract.did
	dfx deploy mu_smart_contract --network "${NETWORK}"

deploy-all: create-canisters deploy-exchange_rate_canister deploy-mu_smart_contract

clean:
	rm -rf .dfx
	rm -rf dist
	rm -rf node_modules
	rm -rf src/declarations
	rm -f .env

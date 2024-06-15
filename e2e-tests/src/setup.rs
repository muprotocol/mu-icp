#![cfg(test)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use candid::decode_one;
use candid::encode_one;
use candid::CandidType;
use candid::Encode;
use candid::Nat;
use candid::Principal;
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk::api::management_canister::provisional::CanisterIdRecord;
use ic_ledger_types::AccountBalanceArgs;
use ic_ledger_types::AccountIdentifier;
use ic_ledger_types::Memo;
use ic_ledger_types::Subaccount;
use ic_ledger_types::Tokens;
use ic_ledger_types::TransferArgs;
use ic_ledger_types::TransferResult;
use ic_ledger_types::DEFAULT_FEE;
use ic_ledger_types::DEFAULT_SUBACCOUNT;
use pocket_ic::call_candid_as;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::common::rest::SubnetId;
use pocket_ic::CanisterSettings;
use pocket_ic::WasmResult;
use pocket_ic::{PocketIc, PocketIcBuilder};
use serde::Deserialize;
use serde::Serialize;

use crate::canisters::ledger_canister;
use crate::canisters::mu_smart_contract;
use crate::canisters::mu_smart_contract::Result_;
use crate::utils::random_principal;

const MU_SMART_CONTRACT_WASM_FILE: OnceLock<Vec<u8>> = OnceLock::new();
// 2T cycles
const INIT_CYCLES: u128 = 2_000_000_000_000;

pub fn canister_wasm_file(name: &str, target: &str) -> Vec<u8> {
    let subpath_to_wasm_module = format!("wasm32-unknown-unknown/{target}/{name}.wasm");

    let file_path = if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
    } else {
        PathBuf::from("../target/")
    }
    .join(subpath_to_wasm_module);

    std::fs::read(file_path).unwrap()
}

fn mu_smart_contract_wasm_file() -> Vec<u8> {
    MU_SMART_CONTRACT_WASM_FILE
        .get_or_init(|| canister_wasm_file("mu_smart_contract", "canister-release"))
        .clone()
}

pub struct TestCase {
    pub pic: PocketIc,
    pub mu_smart_contract: Principal,
    pub ledger_canister: Principal,
    pub developer1: Principal,
}

impl TestCase {
    pub fn setup() -> Self {
        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_application_subnet()
            .build();
        let nns_subnet = pic.topology().get_nns().unwrap();
        let app_subnet = pic.topology().get_app_subnets()[0];

        let proxy_canister = pic.create_canister_on_subnet(None, None, app_subnet);
        pic.add_cycles(proxy_canister, INIT_CYCLES);

        let ledger_wasm = std::fs::read("../nns-modules/ledger.wasm").unwrap();
        let ledger_canister = create_canister_on_subnet_with_id(
            &pic,
            None,
            None,
            nns_subnet,
            ic_ledger_types::MAINNET_LEDGER_CANISTER_ID,
        );

        // Give the proxy canister some tokens to pay the beneficiary.
        let developer1 = random_principal();
        let developer1_ledger = AccountIdentifier::new(&developer1, &DEFAULT_SUBACCOUNT);

        let mut initial_balances = HashMap::new();
        initial_balances.insert(
            developer1_ledger,
            ledger_canister::Tokens {
                e8s: 10_000_000_000,
            },
        );
        initial_balances.insert(
            AccountIdentifier::new(&proxy_canister, &DEFAULT_SUBACCOUNT),
            ledger_canister::Tokens { e8s: 10_000_000 },
        );

        // Specify token details.
        let ledger_canister_init_args = crate::canisters::ledger_canister::InitArgs {
            token_symbol: Some("ICP".to_string()),
            transfer_fee: Some(ledger_canister::Tokens {
                e8s: DEFAULT_FEE.e8s(),
            }),
            minting_account: AccountIdentifier::new(&Principal::anonymous(), &DEFAULT_SUBACCOUNT)
                .to_string(),
            initial_values: initial_balances
                .into_iter()
                .map(|(a, b)| (a.to_string(), b))
                .collect(),
            token_name: Some("ICP".to_string()),
            send_whitelist: vec![],
            maximum_number_of_accounts: None,
            accounts_overflow_trim_quantity: None,
            transaction_window: None,
            max_message_size_bytes: None,
            icrc1_minting_account: None,
            archive_options: None,
            feature_flags: None,
        };

        // Install ledger canister.
        pic.install_canister(
            ledger_canister,
            ledger_wasm,
            encode_one(ledger_canister_init_args).unwrap(),
            None,
        );

        let mu_smart_contract = pic.create_canister_on_subnet(None, None, app_subnet);
        pic.add_cycles(mu_smart_contract, INIT_CYCLES);

        let args = Encode!(&mu_smart_contract::InitArgs {
            minimum_escrow_balance_for_deploy: mu_smart_contract::Tokens { e8s: 1_000_000_000 },
            max_apps_per_developer: 2,
            commition_rate: 0.05,
            exchange_rate_timeout_seconds: 10,
        })
        .unwrap();

        pic.install_canister(mu_smart_contract, mu_smart_contract_wasm_file(), args, None);

        Self {
            pic,
            mu_smart_contract,
            ledger_canister,
            developer1,
        }
    }

    pub fn setup_with_registered_developer1() -> Self {
        let test_case = Self::setup();
        let result = call_candid_as::<_, (Result_,)>(
            &test_case.pic,
            test_case.mu_smart_contract,
            RawEffectivePrincipal::None,
            test_case.developer1,
            "register_developer",
            ((),),
        )
        .unwrap();

        assert_eq!(Result_::Ok(test_case.developer1), result.0);
        test_case
    }

    pub fn ledger_balance_of(&self, account: AccountIdentifier) -> Tokens {
        let args = encode_one(AccountBalanceArgs { account }).unwrap();
        let result = self
            .pic
            .query_call(
                self.ledger_canister,
                Principal::anonymous(),
                "account_balance",
                args,
            )
            .unwrap();
        match result {
            WasmResult::Reply(r) => decode_one(&r).unwrap(),
            WasmResult::Reject(e) => panic!("{e}"),
        }
    }

    pub fn ledger_transfer(
        &self,
        caller: Principal,
        from: Option<Subaccount>,
        to: AccountIdentifier,
        amount: Tokens,
    ) -> TransferResult {
        let args = encode_one(TransferArgs {
            memo: Memo(0),
            amount,
            fee: DEFAULT_FEE,
            from_subaccount: from,
            to,
            created_at_time: None,
        })
        .unwrap();

        let result = self
            .pic
            .update_call(self.ledger_canister, caller, "transfer", args)
            .unwrap();
        match result {
            WasmResult::Reply(r) => decode_one(&r).unwrap(),
            WasmResult::Reject(e) => panic!("{e}"),
        }
    }
}

pub fn create_canister_on_subnet_with_id(
    pic: &PocketIc,
    sender: Option<Principal>,
    settings: Option<CanisterSettings>,
    subnet_id: SubnetId,
    specified_id: Principal,
) -> CanisterId {
    let CanisterIdRecord { canister_id } = call_candid_as(
        pic,
        Principal::management_canister(),
        RawEffectivePrincipal::SubnetId(subnet_id.as_slice().to_vec()),
        sender.unwrap_or(Principal::anonymous()),
        "provisional_create_canister_with_cycles",
        (ProvisionalCreateCanisterArgument {
            settings,
            amount: Some(0_u64.into()),
            specified_id: Some(specified_id),
        },),
    )
    .map(|(x,)| x)
    .unwrap();
    canister_id
}

#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
struct ProvisionalCreateCanisterArgument {
    pub settings: Option<CanisterSettings>,
    pub specified_id: Option<Principal>,
    pub amount: Option<Nat>,
}

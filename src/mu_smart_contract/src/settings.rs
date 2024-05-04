use crate::memory::STATE;
use candid::CandidType;
use candid::Deserialize;
use ic_ledger_types::Tokens;
use std::time::Duration;

#[derive(Debug)]
pub struct Settings {
    pub minimum_escrow_balance_for_deploy: Tokens,
    pub max_apps_per_developer: usize,
    pub commition_rate: f32,
    pub exchange_rate_timeout: Duration,
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    pub minimum_escrow_balance_for_deploy: Tokens,
    pub max_apps_per_developer: usize,
    pub commition_rate: f32,
    pub exchange_rate_timeout_seconds: u64,
}

#[ic_cdk::init]
fn init_canister(init_args: crate::settings::InitArgs) {
    let settings = Settings {
        minimum_escrow_balance_for_deploy: init_args.minimum_escrow_balance_for_deploy,
        max_apps_per_developer: init_args.max_apps_per_developer,
        commition_rate: init_args.commition_rate,
        exchange_rate_timeout: Duration::from_secs(init_args.exchange_rate_timeout_seconds),
    };

    STATE.with_borrow_mut(|s| s.init_settings(settings));
}

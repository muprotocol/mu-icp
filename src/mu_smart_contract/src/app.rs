use std::borrow::Cow;

use candid::CandidType;
use candid::Decode;
use candid::Deserialize;
use candid::Encode;
use candid::Principal;
use ic_cdk::api::management_canister::main::raw_rand;
use ic_ledger_types::Timestamp;
use ic_ledger_types::Tokens;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde_bytes::ByteBuf;

use crate::developer::Developer;
use crate::developer::DeveloperID;
use crate::error::Error;
use crate::memory::STATE;
use crate::utils::exchange::top_up_canister;
use crate::Result;

#[derive(CandidType, Deserialize, Clone)]
pub enum UsageKind {
    CyclesCharge { cylces: u128 },
    AdditionalServices { details: ByteBuf }, // Will be filled in future
}

#[derive(CandidType, Deserialize)]
pub struct AppUsage {
    kind: UsageKind,
    timestamp: Timestamp,
    amount: Tokens,
    is_paid: bool,
}

pub type AppID = Principal;

#[derive(CandidType, Deserialize)]
pub struct App {
    // I know this is not good, but we need a way to link back this app to the developer.
    pub developer_id: DeveloperID,
    pub state: AppState,
    pub usages: Vec<AppUsage>,
}

impl App {
    pub fn as_dto(&self, id: AppID) -> crate::app::dto::AppDto {
        let state = match self.state {
            AppState::Active(ref app) => dto::AppState::Active {
                revision: app.revision,
                name: app.name.clone(),
            },
            AppState::Deleted => dto::AppState::Deleted,
        };

        let usages = self
            .usages
            .iter()
            .map(|u| dto::AppUsage {
                kind: u.kind.clone(),
                timestamp: u.timestamp,
                amount: u.amount,
            })
            .collect();
        dto::AppDto { id, state, usages }
    }
}

#[derive(CandidType, Deserialize)]
pub enum AppState {
    Active(ActiveApp),
    Deleted,
}

#[derive(CandidType, Deserialize)]
pub struct ActiveApp {
    pub revision: u32,
    pub name: String,
    pub data: Vec<u8>,
}

impl Storable for App {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[ic_cdk::query]
fn get_app(app_id: crate::app::AppID) -> Result<Option<crate::app::dto::AppDto>> {
    let (developer_id, _) = Developer::get_caller_developer_account()?;
    STATE.with_borrow(|s| {
        s.get_app_of_developer(&developer_id, &app_id)
            .map(|apps| apps.map(|i| i.as_dto(app_id)))
    })
}

#[ic_cdk::query]
fn get_apps() -> Result<Vec<crate::app::dto::AppDto>> {
    let (developer_id, _) = Developer::get_caller_developer_account()?;
    STATE.with_borrow(|s| {
        s.get_apps_of_developer(&developer_id).map(|i| {
            i.into_iter()
                .map(|(app_id, app)| app.as_dto(app_id))
                .collect()
        })
    })
}

// Note: Will not deploy, just upload for now.
#[ic_cdk::update]
async fn deploy_app(request: crate::app::dto::DeployAppRequest) -> Result<crate::app::AppID> {
    let (developer_id, developer) = Developer::get_caller_developer_account()?;
    developer
        .ensure_developer_escorw_has_minimum_balance_for_deploy()
        .await?;
    developer.ensure_developer_has_budget_for_new_app()?;

    // TODO: Check if there is already another app deployed with this id and regenerate or change
    // it if needed.
    let app_id = {
        let rand_bytes = raw_rand().await.map_err(|(_, s)| Error::Internal(s))?.0;
        Principal::from_slice(&rand_bytes[0..29])
    };

    let app = App {
        developer_id,
        state: AppState::Active(ActiveApp {
            revision: 1,
            name: request.name,
            data: request.app_data,
        }),
        usages: Vec::new(),
    };

    STATE.with_borrow_mut(|s| s.register_app(app_id, app));

    Ok(app_id)
}

// Note: Will not undeploy, just remove for now.
#[ic_cdk::update]
fn remove_app(app_id: crate::app::AppID) -> Result<()> {
    let _ = Developer::get_caller_developer_account()?;
    STATE.with_borrow_mut(|s| s.remove_app(app_id))
}

// Specific for canisters to request more cycles transferred to them.
#[ic_cdk::update]
async fn request_cycles(cycles: u64) -> Result<u128> {
    let app_id = ic_cdk::caller();
    let escrow_account = STATE.with_borrow(|s| {
        let app = s.get_app(&app_id)?;
        let developer = s.get_developer(&app.developer_id)?;
        Ok(developer.escrow_account)
    })?;
    let (cycles_topped_up, icp_tokens_used) =
        top_up_canister(escrow_account, app_id, cycles).await?;

    let usage = AppUsage {
        kind: UsageKind::CyclesCharge {
            cylces: cycles_topped_up,
        },
        timestamp: Timestamp {
            timestamp_nanos: ic_cdk::api::time(),
        },
        amount: icp_tokens_used,
        is_paid: true,
    };

    STATE
        .with_borrow_mut(|s| s.register_usage(app_id, usage))
        .unwrap();

    Ok(cycles_topped_up)
}

pub mod dto {
    use super::*;

    #[derive(CandidType, Deserialize)]
    pub(super) enum AppState {
        Active { revision: u32, name: String },
        Deleted,
    }

    #[derive(CandidType, Deserialize)]
    pub(super) struct AppUsage {
        pub kind: UsageKind,
        pub timestamp: Timestamp,
        pub amount: Tokens,
    }

    #[derive(CandidType, Deserialize)]
    pub struct AppDto {
        pub id: AppID,
        pub(super) state: AppState,
        pub(super) usages: Vec<AppUsage>,
    }

    #[derive(CandidType, Deserialize)]
    pub struct DeployAppRequest {
        pub name: String,
        pub app_data: Vec<u8>,
    }
}

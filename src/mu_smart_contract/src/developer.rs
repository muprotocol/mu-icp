use std::borrow::Cow;

use candid::CandidType;
use candid::Decode;
use candid::Deserialize;
use candid::Encode;
use candid::Principal;
use ic_cdk::api::management_canister::main::raw_rand;
use ic_ledger_types::AccountIdentifier;
use ic_ledger_types::Memo;
use ic_ledger_types::Subaccount;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;

use crate::app::AppID;
use crate::error::Error;
use crate::memory::STATE;
use crate::utils::get_developer_escrow_balance;
use crate::utils::transfer_tokens;
use crate::Result;

pub type DeveloperID = Principal;

#[derive(CandidType, Deserialize)]
pub struct Developer {
    pub(crate) escrow_account: Subaccount,
    pub(crate) apps: Vec<AppID>,
}

impl Developer {
    pub fn as_dto(&self) -> crate::developer::dto::DeveloperDto {
        dto::DeveloperDto {
            escrow_account: AccountIdentifier::new(&ic_cdk::id(), &self.escrow_account),
        }
    }

    pub fn get_caller_developer_account() -> Result<(DeveloperID, Developer)> {
        let developer_id = ic_cdk::caller();
        STATE
            .with_borrow(|s| s.get_developer(&developer_id))
            .map(|d| (developer_id, d))
    }

    pub fn ensure_developer_account_does_not_exist() -> Result<DeveloperID> {
        let developer_id = ic_cdk::caller();
        match STATE.with_borrow(|s| s.get_developer(&developer_id)) {
            Err(Error::DeveloperAccountNotFound) => Ok(developer_id),
            Err(e) => Err(e),
            Ok(_) => Err(Error::DeveloperAccountAlreadyExist),
        }
    }

    pub async fn ensure_developer_escorw_has_minimum_balance_for_deploy(&self) -> Result<()> {
        let escrow_balance = get_developer_escrow_balance(&self.escrow_account).await?;
        let minimum_escrow_balance_for_deploy =
            STATE.with_borrow(|s| s.settings().minimum_escrow_balance_for_deploy);

        if escrow_balance < minimum_escrow_balance_for_deploy {
            Err(Error::InsufficientBalanceForDeploy {
                was: escrow_balance,
                needed: minimum_escrow_balance_for_deploy,
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_developer_has_budget_for_new_app(&self) -> Result<()> {
        let apps_count = self.apps.len();
        if apps_count > STATE.with_borrow(|s| s.settings().max_apps_per_developer) {
            Err(Error::MaxAppsCountReached)
        } else {
            Ok(())
        }
    }
}

#[ic_cdk::update]
async fn register_developer() -> Result<crate::developer::DeveloperID> {
    let developer_id = Developer::ensure_developer_account_does_not_exist()?;

    // TODO: Check if this subaccount is already in use and regenerate or change it if needed.
    let escrow_account = raw_rand()
        .await
        .map_err(|(_, s)| Error::Internal(s))?
        .0
        .try_into()
        .map(Subaccount)
        .unwrap();

    let developer = Developer {
        escrow_account,
        apps: Vec::new(),
    };

    let _ = STATE.with_borrow_mut(|s| s.register_developer(developer_id, developer));
    Ok(developer_id)
}

#[ic_cdk::query]
fn get_developer() -> Result<crate::developer::dto::DeveloperDto> {
    let (_, developer) = Developer::get_caller_developer_account()?;
    Ok(developer.as_dto())
}

#[ic_cdk::update]
async fn request_escrow_withdraw(
    to: ic_ledger_types::AccountIdentifier,
    amount: ic_ledger_types::Tokens,
) -> Result<ic_ledger_types::BlockIndex> {
    let (_, developer) = Developer::get_caller_developer_account()?;
    transfer_tokens(developer.escrow_account, to, amount, Memo(0)).await
}

impl Storable for Developer {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub mod dto {
    use super::*;

    #[derive(CandidType, Deserialize)]
    pub struct DeveloperDto {
        pub escrow_account: AccountIdentifier,
    }
}

use std::time::Instant;

use crate::app::AppID;
use crate::declarations::exchange_rate_canister as exchange;

use crate::error::Error;
use crate::memory::STATE;
use crate::utils::transfer_tokens;
use crate::Result;

use candid::CandidType;
use candid::Principal;
use exchange::Asset;
use exchange::AssetClass;
use exchange::GetExchangeRateRequest;
use exchange::GetExchangeRateResult;
use ic_ledger_types::AccountIdentifier;
use ic_ledger_types::BlockIndex;
use ic_ledger_types::Memo;
use ic_ledger_types::Subaccount;
use ic_ledger_types::Tokens;
use serde::Deserialize;

pub const MEMO_TOP_UP_CANISTER: u64 = 1347768404_u64;
pub const MAINNET_CYCLE_MINTER_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01, 0x01]);
const NOTIFY_TOP_UP_METHOD: &str = "notify_top_up";

async fn icp_cycles_exchange_rate() -> Result<u64> {
    let request = GetExchangeRateRequest {
        quote_asset: Asset {
            symbol: "Cycles".to_string(),
            class: AssetClass::Cryptocurrency,
        },
        base_asset: Asset {
            symbol: "ICP".to_string(),
            class: AssetClass::Cryptocurrency,
        },
        timestamp: None,
    };

    let response = ic_cdk::api::call::call_with_payment::<_, (GetExchangeRateResult,)>(
        exchange::CANISTER_ID,
        "get_exchange_rate",
        (request,),
        1_000_000_000, // 1B cycles should be sent with each request.
    )
    .await
    .map_err(|e| {
        Error::Internal(format!(
            "Failed to fetch exchange rate, error_code: {:?}, reason: {}",
            e.0, e.1
        ))
    })?
    .0;

    match response {
        GetExchangeRateResult::Ok(r) => Ok(r.rate),
        GetExchangeRateResult::Err(e) => Err(Error::Internal(format!(
            "Failed to fetch exchange rate, error: {e:?}"
        ))),
    }
}

/// Get exchange rate of ICP token to Cycles
async fn get_and_update_icp_cycles_exchange_rate() -> Result<u64> {
    async fn renew() -> Result<u64> {
        let rate = crate::utils::exchange::icp_cycles_exchange_rate().await?;
        STATE.with_borrow_mut(|s| {
            s.icp_cycles_exchange_rate =
                Some((rate, Instant::now() + s.settings().exchange_rate_timeout));
        });
        Ok(rate)
    }

    match STATE.with_borrow(|s| s.icp_cycles_exchange_rate) {
        Some((_, timeout)) if Instant::now() > timeout => renew().await,
        None => renew().await,
        Some((rate, _)) => Ok(rate),
    }
}

pub async fn top_up_canister(
    from: Subaccount,
    app_id: AppID,
    amount: u64,
) -> Result<(u128, Tokens)> {
    let rate = get_and_update_icp_cycles_exchange_rate().await?;
    let icp_needed = Tokens::from_e8s((amount / rate) * Tokens::SUBDIVIDABLE_BY);

    let memo = Memo(MEMO_TOP_UP_CANISTER);
    let to = AccountIdentifier::new(&MAINNET_CYCLE_MINTER_CANISTER_ID, &Subaccount::from(app_id));

    let block_index = transfer_tokens(from, to, icp_needed, memo).await?;
    let cycles = notify_top_up(app_id, block_index).await?;
    Ok((cycles, icp_needed))
}

#[derive(CandidType)]
struct NotifyTopUpArg {
    pub block_index: BlockIndex,
    pub canister_id: Principal,
}

type NotifyTopUpResult = std::result::Result<u128, NotifyError>;

#[derive(CandidType, Deserialize, Debug)]
enum NotifyError {
    Refunded {
        reason: String,
        block_index: Option<BlockIndex>,
    },
    Processing,
    TransactionTooOld(BlockIndex),
    InvalidTransaction(String),
    Other {
        error_code: u64,
        error_message: String,
    },
}

async fn notify_top_up(canister_id: Principal, block_index: BlockIndex) -> Result<u128> {
    let args = NotifyTopUpArg {
        block_index,
        canister_id,
    };

    ic_cdk::call::<_, (NotifyTopUpResult,)>(
        MAINNET_CYCLE_MINTER_CANISTER_ID,
        NOTIFY_TOP_UP_METHOD,
        (args,),
    )
    .await
    .map_err(|e| {
        Error::Internal(format!(
            "Failed to send notify top-up message, error_code: {:?}, reason: {}",
            e.0, e.1
        ))
    })?
    .0
    .map_err(|e| Error::Internal(format!("Notify top-up failed, error_code: {e:?}",)))
}

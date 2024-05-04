use ic_ledger_types::account_balance;
use ic_ledger_types::transfer;
use ic_ledger_types::AccountBalanceArgs;
use ic_ledger_types::AccountIdentifier;
use ic_ledger_types::BlockIndex;
use ic_ledger_types::Memo;
use ic_ledger_types::Subaccount;
use ic_ledger_types::Tokens;
use ic_ledger_types::TransferArgs;
use ic_ledger_types::DEFAULT_FEE;
use ic_ledger_types::MAINNET_LEDGER_CANISTER_ID;

use crate::error::Error;
use crate::Result;

pub mod exchange;

pub async fn get_developer_escrow_balance(subaccount: &Subaccount) -> Result<Tokens> {
    let args = AccountBalanceArgs {
        account: AccountIdentifier::new(&ic_cdk::id(), subaccount),
    };

    account_balance(MAINNET_LEDGER_CANISTER_ID, args)
        .await
        .map_err(|e| {
            Error::Internal(format!(
                "Failed to fetch account balance, error_code: {:?}, reason: {}",
                e.0, e.1
            ))
        })
}

pub async fn transfer_tokens(
    from_subaccount: Subaccount,
    to: AccountIdentifier,
    amount: Tokens,
    memo: Memo,
) -> Result<BlockIndex> {
    let args = TransferArgs {
        memo,
        amount,
        fee: DEFAULT_FEE,
        from_subaccount: Some(from_subaccount),
        to,
        created_at_time: None,
    };

    transfer(MAINNET_LEDGER_CANISTER_ID, args)
        .await
        .map_err(|e| {
            Error::Internal(format!(
                "Failed to call transfer on ic_ledger, error_code: {:?}, reason: {}",
                e.0, e.1
            ))
        })?
        .map_err(|e| Error::Internal(format!("Transfer failed, reason: {e}")))
}

use candid::CandidType;
use ic_ledger_types::Tokens;

#[derive(CandidType, Debug)]
pub enum Error {
    Internal(String),
    AppNotFound,
    DeveloperAccountNotFound,
    DeveloperAccountAlreadyExist,
    MaxAppsCountReached,
    InsufficientBalanceForDeploy { was: Tokens, needed: Tokens },
}

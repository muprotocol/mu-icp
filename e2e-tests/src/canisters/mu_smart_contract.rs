// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk::api::call::CallResult as Result;

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub struct Tokens { pub e8s: u64 }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub struct InitArgs {
  pub exchange_rate_timeout_seconds: u64,
  pub minimum_escrow_balance_for_deploy: Tokens,
  pub commition_rate: f32,
  pub max_apps_per_developer: u64,
}

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub struct DeployAppRequest {
  pub name: String,
  pub app_data: serde_bytes::ByteBuf,
}

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum Error {
  Internal(String),
  DeveloperAccountNotFound,
  MaxAppsCountReached,
  AppNotFound,
  DeveloperAccountAlreadyExist,
  InsufficientBalanceForDeploy{ was: Tokens, needed: Tokens },
}

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum Result_ { Ok(Principal), Err(Error) }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum UsageKind {
  AdditionalServices{ details: serde_bytes::ByteBuf },
  CyclesCharge{ cylces: candid::Nat },
}

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub struct Timestamp { pub timestamp_nanos: u64 }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub struct AppUsage {
  pub kind: UsageKind,
  pub timestamp: Timestamp,
  pub amount: Tokens,
}

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum AppState { Active{ name: String, revision: u32 }, Deleted }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub struct AppDto {
  pub id: Principal,
  pub usages: Vec<AppUsage>,
  pub state: AppState,
}

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum GetAppResult { Ok(Option<AppDto>), Err(Error) }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum GetAppsResult { Ok(Vec<AppDto>), Err(Error) }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub struct DeveloperDto { pub escrow_account: serde_bytes::ByteBuf }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum GetDeveloperResult { Ok(DeveloperDto), Err(Error) }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum RemoveAppResult { Ok, Err(Error) }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum RequestCyclesResult { Ok(candid::Nat), Err(Error) }

#[derive(Debug, PartialEq, CandidType, Deserialize)]
pub enum RequestEscrowWithdrawResult { Ok(u64), Err(Error) }

pub struct MuSmartContract(pub Principal);
impl MuSmartContract {
  pub async fn deploy_app(&self, arg0: DeployAppRequest) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "deploy_app", (arg0,)).await
  }
  pub async fn get_app(&self, arg0: Principal) -> Result<(GetAppResult,)> {
    ic_cdk::call(self.0, "get_app", (arg0,)).await
  }
  pub async fn get_apps(&self) -> Result<(GetAppsResult,)> {
    ic_cdk::call(self.0, "get_apps", ()).await
  }
  pub async fn get_developer(&self) -> Result<(GetDeveloperResult,)> {
    ic_cdk::call(self.0, "get_developer", ()).await
  }
  pub async fn register_developer(&self) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "register_developer", ()).await
  }
  pub async fn remove_app(&self, arg0: Principal) -> Result<
    (RemoveAppResult,)
  > { ic_cdk::call(self.0, "remove_app", (arg0,)).await }
  pub async fn request_cycles(&self, arg0: u64) -> Result<
    (RequestCyclesResult,)
  > { ic_cdk::call(self.0, "request_cycles", (arg0,)).await }
  pub async fn request_escrow_withdraw(
    &self,
    arg0: serde_bytes::ByteBuf,
    arg1: Tokens,
  ) -> Result<(RequestEscrowWithdrawResult,)> {
    ic_cdk::call(self.0, "request_escrow_withdraw", (arg0,arg1,)).await
  }
}
pub const CANISTER_ID : Principal = Principal::from_slice(&[128, 0, 0, 0, 0, 16, 0, 2, 1, 1]); // bd3sg-teaaa-aaaaa-qaaba-cai
pub const mu_smart_contract : MuSmartContract = MuSmartContract(CANISTER_ID);
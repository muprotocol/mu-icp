use ic_cdk_bindgen::{Builder, Config};

fn main() {
    let mut builder = Builder::new();

    let mut mu_smart_contract_canister = Config::new("mu_smart_contract");
    mu_smart_contract_canister
        .binding
        .set_type_attributes("#[derive(Debug, PartialEq, CandidType, Deserialize)]".into());

    builder.add(mu_smart_contract_canister);

    let mut ledger_canister = Config::new("ledger_canister");
    ledger_canister
        .binding
        .set_type_attributes("#[derive(Debug, CandidType, Deserialize)]".into());

    builder.add(ledger_canister);

    builder.build(None); // default write to src/declarations
}

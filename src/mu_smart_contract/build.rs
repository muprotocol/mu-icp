use ic_cdk_bindgen::{Builder, Config};

fn main() {
    let mut builder = Builder::new();

    let mut exchange_rate_canister = Config::new("exchange_rate_canister");
    exchange_rate_canister
        .binding
        .set_type_attributes("#[derive(Debug, PartialEq, CandidType, Deserialize)]".into());

    builder.add(exchange_rate_canister);

    builder.build(None); // default write to src/declarations
}

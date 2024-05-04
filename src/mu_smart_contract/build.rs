use ic_cdk_bindgen::{Builder, Config};
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
    let mut builder = Builder::new();

    let mut exchange_rate_canister = Config::new("exchange_rate_canister");
    exchange_rate_canister
        .binding
        .set_type_attributes("#[derive(Debug, CandidType, Deserialize)]".into());

    builder.add(exchange_rate_canister);
    builder.build(Some(manifest_dir.join("src/declarations"))); // default write to src/declarations
}

type Result<T> = std::result::Result<T, error::Error>;

mod app;
mod declarations;
mod developer;
mod error;
mod memory;
pub mod settings;
mod utils;

ic_cdk::export_candid!();

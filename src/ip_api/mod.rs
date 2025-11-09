/// ip-api client and helpers.
///
/// The implementation lives in `client.rs` and the module re-exports the
/// public functions so callers continue to use `crate::ip_api::get_isp`.

pub mod client;

pub use client::{get_isp, get_isp_with_client_and_url};

mod response;
pub mod server;

pub use server::start_metrics_server;

// Re-export response building helpers for unit tests and potential reuse.
pub(crate) use response::{
    build_health_response, build_metrics_response, build_not_found_response,
};

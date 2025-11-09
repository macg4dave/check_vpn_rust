/// Simple logging setup wrapper. Keeps `main.rs` tidy and centralizes logging initialization.
pub fn init() {
    env_logger::init();
}

use std::process::ExitCode;

use check_vpn::error_types::AppError;

#[test]
fn exit_code_mapping_for_app_errors() {
    // Config validation error -> EXIT_INVALID_CONFIG
    let cfg_err = check_vpn::config::validation::ValidationErrors(vec!["bad".into()]);
    let e = AppError::ConfigError(cfg_err);
    let c = check_vpn::error_types::exit_code_for_error(&e);
    assert_eq!(
        c,
        ExitCode::from(check_vpn::config::EXIT_INVALID_CONFIG as u8)
    );

    // Config load error -> same mapping
    let e = AppError::ConfigLoadError(anyhow::anyhow!("bad config"));
    let c = check_vpn::error_types::exit_code_for_error(&e);
    assert_eq!(
        c,
        ExitCode::from(check_vpn::config::EXIT_INVALID_CONFIG as u8)
    );

    // Runtime -> generic 1
    let e = AppError::RuntimeError(anyhow::anyhow!("runtime"));
    let c = check_vpn::error_types::exit_code_for_error(&e);
    assert_eq!(c, ExitCode::from(1));

    // Action failed -> mapping 3
    let e = AppError::ActionFailed(anyhow::anyhow!("action fail"));
    let c = check_vpn::error_types::exit_code_for_error(&e);
    assert_eq!(c, ExitCode::from(3));

    // Args error -> mapping 4
    let e = AppError::ArgsError(anyhow::anyhow!("arg"));
    let c = check_vpn::error_types::exit_code_for_error(&e);
    assert_eq!(c, ExitCode::from(4));

    // Other -> falls back to 1
    let e = AppError::Other(anyhow::anyhow!("other"));
    let c = check_vpn::error_types::exit_code_for_error(&e);
    assert_eq!(c, ExitCode::from(1));
}

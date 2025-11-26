use std::fs;
use std::io::Write;

// Ensure a clean environment for tests that manipulate HOME
fn with_temp_home<F: FnOnce(&tempfile::TempDir)>(f: F) {
    let td = tempfile::tempdir().expect("create tempdir");
    let home = td.path().to_str().unwrap().to_string();
    let orig_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", &home);
    // Run the closure
    f(&td);
    // Restore
    if let Some(h) = orig_home {
        std::env::set_var("HOME", h);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn test_config_lookup_order_user_and_system() {
    with_temp_home(|td| {
        // create a local cwd file and ensure it is preferred over others
        let cwd = std::env::current_dir().unwrap();
        let local_path = cwd.join("check_vpn.xml");
        let mut f = fs::File::create(&local_path).unwrap();
        write!(f, "<config><interval>123</interval></config>").unwrap();

        // Create per-user linux config ~/.config/check_vpn/config.xml
        let cfg_dir = td.path().join(".config").join("check_vpn");
        fs::create_dir_all(&cfg_dir).unwrap();
        let user_cfg = cfg_dir.join("config.xml");
        fs::write(&user_cfg, "<config><interval>456</interval></config>").unwrap();

        // Create macos-style config (Library/Preferences/check_vpn/config.xml)
        let mac_cfg_dir = td.path().join("Library").join("Preferences").join("check_vpn");
        fs::create_dir_all(&mac_cfg_dir).unwrap();
        let mac_cfg = mac_cfg_dir.join("config.xml");
        fs::write(&mac_cfg, "<config><interval>789</interval></config>").unwrap();

        // Create /etc style (simulate by creating a file and setting CHECK_VPN_CONFIG to it later if needed)
        let etc_cfg = td.path().join("etc_check_vpn_config.xml");
        fs::write(&etc_cfg, "<config><interval>999</interval></config>").unwrap();

        // 1) When CHECK_VPN_CONFIG is set, that path wins
        std::env::set_var("CHECK_VPN_CONFIG", etc_cfg.to_str().unwrap());
        let cfg = check_vpn::config::Config::load().expect("load via env");
        assert_eq!(cfg.interval.unwrap(), 999);
        std::env::remove_var("CHECK_VPN_CONFIG");

        // 2) local cwd file should be preferred next
        let cfg = check_vpn::config::Config::load().expect("load local");
        assert_eq!(cfg.interval.unwrap(), 123);

        // remove local so next lookup hits per-user locations
        fs::remove_file(&local_path).unwrap();

        // Depending on target_os cfg in the build, either macOS or linux path will be checked.
        // We assert that at least one of the per-user files is read if present by temporarily
        // calling load and matching interval against one of the known values.
        let cfg = check_vpn::config::Config::load().expect("load per-user or etc");
        let val = cfg.interval.unwrap();
        assert!(val == 456 || val == 789 || val == 999);
    });
}

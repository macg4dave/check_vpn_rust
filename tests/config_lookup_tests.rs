use check_vpn::config::{Config, ConfigSource};
use serial_test::serial;
use std::path::PathBuf;
mod support;

struct WorkingDirectory(PathBuf);
impl WorkingDirectory {
    fn enter(path: &std::path::Path) -> Self {
        let guard = Self(std::env::current_dir().unwrap());
        std::env::set_current_dir(path).unwrap();
        guard
    }
}
impl Drop for WorkingDirectory {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.0).unwrap();
    }
}

#[test]
#[serial(env)]
fn lookup_priority_and_discovered_source_are_stable() {
    let dir = tempfile::tempdir().unwrap();
    let _cwd = WorkingDirectory::enter(dir.path());
    let _home = support::ScopedEnv::set_str("HOME", dir.path().to_str().unwrap());
    let _xdg = support::ScopedEnv::set("XDG_CONFIG_HOME", None);
    let _env = support::ScopedEnv::set("CHECK_VPN_CONFIG", None);
    let local = dir.path().join("check_vpn.xml");
    std::fs::write(&local, "<config><interval>123</interval></config>").unwrap();
    let env_path = dir.path().join("override.xml");
    std::fs::write(&env_path, "<config><interval>999</interval></config>").unwrap();
    {
        let _override =
            support::ScopedEnv::set("CHECK_VPN_CONFIG", Some(env_path.into_os_string()));
        assert_eq!(Config::load().unwrap().interval, Some(999));
        assert_eq!(
            ConfigSource::resolve(Some(&local))
                .unwrap()
                .load()
                .unwrap()
                .interval,
            Some(123)
        );
    }
    let source = ConfigSource::resolve(None).unwrap();
    assert_eq!(source.load().unwrap().interval, Some(123));
    std::fs::remove_file(&local).unwrap();
    assert!(
        source.load().is_err(),
        "disappearance must not switch to defaults"
    );
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        #[cfg(target_os = "linux")]
        let user = dir.path().join(".config/check_vpn/config.xml");
        #[cfg(target_os = "macos")]
        let user = dir.path().join("Library/Preferences/check_vpn/config.xml");
        std::fs::create_dir_all(user.parent().unwrap()).unwrap();
        std::fs::write(&user, "<config><interval>456</interval></config>").unwrap();
        assert_eq!(Config::load().unwrap().interval, Some(456));
    }
}

#[cfg(target_os = "macos")]
#[test]
#[serial(env)]
fn configuration_discovery_does_not_write_files() {
    let dir = tempfile::tempdir().unwrap();
    let _cwd = WorkingDirectory::enter(dir.path());
    let _home = support::ScopedEnv::set("HOME", Some(dir.path().as_os_str().to_os_string()));
    let _env = support::ScopedEnv::set("CHECK_VPN_CONFIG", None);
    let _ = Config::load().unwrap();
    assert!(!dir
        .path()
        .join("Library/Preferences/check_vpn/config.xml")
        .exists());
}

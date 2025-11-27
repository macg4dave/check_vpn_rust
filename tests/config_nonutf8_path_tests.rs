#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

#[test]
#[cfg(unix)]
fn load_from_path_handles_non_utf8_paths_without_panicking() {
    use std::fs;
    use std::ffi::OsString;

    // Create a filename containing invalid UTF-8 bytes (0xff)
    let fname = OsString::from_vec(b"cfg-\xff-not-utf8.xml".to_vec());
    let td = tempfile::tempdir().expect("tempdir");
    let mut path = td.path().to_path_buf();
    path.push(&fname);

    // Write a malformed file so load_from_path would fail but must not panic.
    fs::write(&path, "Not XML").expect("write malformed data");

    // Call load_from_path with a Path that may contain non-UTF8; should return Err
    let res = check_vpn::config::Config::load_from_path(path.as_path());
    assert!(res.is_err(), "expected load_from_path to return Err for malformed/non-UTF8 path");
}

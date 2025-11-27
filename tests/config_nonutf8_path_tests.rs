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
    // Some OS/filesystems (notably macOS) will refuse to create filenames
    // containing certain invalid byte sequences and return EILSEQ / "Illegal
    // byte sequence". In that case skip the test rather than failing.
    match fs::write(&path, "Not XML") {
        Ok(()) => {}
        Err(e) => {
            if e.raw_os_error() == Some(92) {
                // EILSEQ on macOS / BSD: cannot create the file with these bytes.
                eprintln!(
                    "skipping test: platform refused to create non-UTF8 filename: {}",
                    e
                );
                return;
            }
            panic!("write malformed data: {}", e);
        }
    }

    // Call load_from_path with a Path that may contain non-UTF8; should return Err
    let res = check_vpn::config::Config::load_from_path(path.as_path());
    assert!(res.is_err(), "expected load_from_path to return Err for malformed/non-UTF8 path");
}

/// JSON I/O helpers.
///
/// This module provides a small, well-documented set of helpers for reading
/// and writing JSON using either in-memory strings, streaming readers/writers,
/// or filesystem paths. The implementation is split into focused submodules
/// to make testing and reuse easier.

pub mod fs;
pub mod stream;

// Re-export the commonly used convenience helpers at `crate::json_io::...` so
// existing callers need only change the module path if they were using the
// previous flat `src/json_io.rs` file.
pub use fs::{read_json, read_json_from_file, write_json, write_json_to_file};
pub use stream::{read_json_from_reader, write_json_to_writer};

#[cfg(test)]
mod tests {
    // Basic integration tests that exercise the public API surface.
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct TestStruct {
        a: String,
        b: u32,
    }

    #[test]
    fn roundtrip_read_write_file() {
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path();

        let value = TestStruct { a: "x".into(), b: 42 };
        write_json(&value, path.to_str().unwrap()).expect("write_json");

        let got: TestStruct = read_json(path.to_str().unwrap()).expect("read_json");
        assert_eq!(got, value);
    }

    #[test]
    fn stream_reader_writer_roundtrip() {
        let value = TestStruct { a: "hey".into(), b: 7 };
        let mut buf = Vec::new();
        write_json_to_writer(&value, &mut buf).expect("write to writer");
        let cur = Cursor::new(buf);
        let got: TestStruct = read_json_from_reader(cur).expect("read from reader");
        assert_eq!(got, value);
    }

    #[test]
    fn read_json_from_file_api() {
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path();
        let value = TestStruct { a: "y".into(), b: 99 };
        write_json_to_file(&value, path).expect("write_json_to_file");
        let got: TestStruct = read_json_from_file(path).expect("read_json_from_file");
        assert_eq!(got, value);
    }
}

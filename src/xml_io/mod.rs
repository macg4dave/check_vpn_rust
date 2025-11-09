/// XML I/O helpers.
///
/// This module provides a small set of helpers for reading and writing XML
/// using one of the selectable backends (`xml_quick` or `xml_serde`). The
/// implementation is split so that backend selection and tests are isolated.

pub mod backend;
pub mod fs;

// Re-export the common convenience helpers.
pub use fs::{read_xml, write_xml};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::NamedTempFile;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct TestStruct {
        name: String,
        value: u32,
    }

    #[test]
    fn roundtrip_file() {
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_str().unwrap().to_string();
        let v = TestStruct { name: "x".into(), value: 3 };
        write_xml(&v, &path).expect("write_xml");
        let got: TestStruct = read_xml(&path).expect("read_xml");
        assert_eq!(got, v);
    }

    #[test]
    fn parse_error_includes_path_and_size() {
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_str().unwrap().to_string();
        // write malformed xml
        std::fs::write(&path, "<bad>").expect("write");
        let err = read_xml::<TestStruct>(&path).unwrap_err();
        let s = format!("{}", err);
        assert!(s.contains(&path));
    }
}

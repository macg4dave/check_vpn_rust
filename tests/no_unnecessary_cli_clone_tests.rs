use std::fs;
use std::path::Path;

// This test discourages the two specific, obvious unnecessary clones we
// previously fixed: `args.init_output.clone()` and
// `args.generate_completions.clone()`. Keeping this check intentionally
// narrow avoids false-positives on valid clone sites (e.g. merging args
// into owned config objects).
#[test]
fn no_obvious_args_clone_patterns() {
    // Only scan the crate's `src/` tree to avoid matching within tests or the
    // test itself (which naturally mentions these patterns).
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    // Walk the important code locations only (avoid target/ and vendor dirs).
    let mut occurrences = Vec::new();

    fn walk(dir: &Path, occurrences: &mut Vec<String>) {
        for entry in fs::read_dir(dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let p = entry.path();
            if p.is_dir() {
                // skip target/ and .git
                if let Some(n) = p.file_name().and_then(|s| s.to_str()) {
                    if n == "target" || n == ".git" { continue; }
                }
                walk(&p, occurrences);
            } else if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                if ext == "rs" {
                    let txt = fs::read_to_string(&p).unwrap_or_default();
                    if txt.contains("args.init_output.clone()") || txt.contains("args.generate_completions.clone()") {
                        occurrences.push(p.display().to_string());
                    }
                }
            }
        }
    }

    walk(&root, &mut occurrences);

    if !occurrences.is_empty() {
        panic!("Found forbidden clone patterns in files: {:?}. Please use references (as_ref()/as_deref()) instead of cloning args.<field>.clone().", occurrences);
    }
}

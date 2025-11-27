use std::ffi::OsString;

/// A small RAII guard that sets an environment variable for the duration of the
/// guard and restores the previous value (or removes it) on drop.
pub struct ScopedEnv {
    key: String,
    prev: Option<OsString>,
    removed: bool,
}

impl ScopedEnv {
    /// Set `key` to `value` for the lifetime of the returned guard.
    /// If `value` is None the variable is removed for the lifetime.
    pub fn set<K: Into<String>, V: Into<Option<OsString>>>(key: K, value: V) -> ScopedEnv {
        let key_s = key.into();
        let prev = std::env::var_os(&key_s);
        match value.into() {
            Some(v) => std::env::set_var(&key_s, &v),
            None => std::env::remove_var(&key_s),
        }
        ScopedEnv {
            key: key_s,
            prev,
            removed: false,
        }
    }

    /// Convenience to set a string value.
    pub fn set_str(key: &str, value: &str) -> ScopedEnv {
        ScopedEnv::set(key, Some(OsString::from(value)))
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        if let Some(prev) = self.prev.take() {
            std::env::set_var(&self.key, prev);
        } else if !self.removed {
            // No previous value: remove it if still present
            std::env::remove_var(&self.key);
        }
    }
}

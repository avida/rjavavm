#![cfg(test)]

use std::ffi::OsString;
use std::env;

/// Test helper: set an env var from `.cargo/config.toml` (if present) and restore
/// the previous value when dropped.
pub struct EnvGuard {
    key: String,
    prev: Option<OsString>,
}

fn read_classpath_from_cargo_config() -> Option<String> {
    let p = std::path::Path::new(".cargo/config.toml");
    let content = std::fs::read_to_string(p).ok()?;
    for line in content.lines() {
        if !line.contains("CLASSPATH") {
            continue;
        }
        if let Some(idx) = line.find("value=\"") {
            let start = idx + "value=\"".len();
            if let Some(end) = line[start..].find('"') {
                return Some(line[start..start + end].to_string());
            }
        }
    }
    None
}

impl EnvGuard {
    pub fn set_from_config(key: &str) -> EnvGuard {
        let prev = env::var_os(key);
        if let Some(cp) = read_classpath_from_cargo_config() {
            unsafe { env::set_var(key, cp); }
        }
        EnvGuard {
            key: key.to_string(),
            prev,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => unsafe { env::set_var(&self.key, v); },
            None => unsafe { env::remove_var(&self.key); },
        }
    }
}

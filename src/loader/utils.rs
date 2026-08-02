pub mod utils {
    use std::env;
    use std::path::{Path, PathBuf};
    use crate::vm::class;

    pub fn lookup_class_file(class_name: &str) -> Option<PathBuf> {
        // Normalize class name to path form (dots to slashes)
        let mut class_path = class_name.replace('.', "/");
        if !class_path.ends_with(".class") {
            class_path.push_str(".class");
        }

        // Check CLASSPATH env var (colon-separated) then current dir
        let classpath = env::var("CLASSPATH").unwrap_or_else(|_| ".".to_string());
        for entry in classpath.split(':') {
            if entry.is_empty() {
                continue;
            }
            let mut p = PathBuf::from(entry);
            p.push(&class_path);
            if p.exists() && p.is_file() {
                return Some(p);
            }
        }

        // Finally try current working directory explicitly
        let cwd = env::current_dir().ok();
        if let Some(mut cwd) = cwd {
            cwd.push(&class_path);
            if cwd.exists() && cwd.is_file() {
                return Some(cwd);
            }
        }

        None
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::env;
        use std::fs;
        use std::path::PathBuf;
        use std::thread;
        use std::time::Duration;

        // Use shared test fixture from crate::test_utils
        use crate::test_utils::EnvGuard;

        fn mk_temp_dir(name: &str) -> PathBuf {
            let mut p = env::temp_dir();
            p.push(format!("rustest_test_{}_{}", name, std::process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            p
        }

        #[test]
        fn finds_in_classpath() {
            let _cfg_guard = EnvGuard::set_from_config("CLASSPATH");
            let dir = "test".to_string();
            unsafe {
                env::set_var("CLASSPATH", dir);
            }
            // give the OS a moment to settle (e.g., filesystem) before lookup
            // thread::sleep(Duration::from_millis(10));

            let found = lookup_class_file("Hello");
            assert!(found.is_some());

            unsafe {
                env::remove_var("CLASSPATH");
            }
        }

        #[test]
        fn returns_none_when_missing() {
            let _cfg_guard = EnvGuard::set_from_config("CLASSPATH");
            unsafe {
                env::set_var("CLASSPATH", "/non/existent/path");
            }
            let found = lookup_class_file("does.not.Exist");
            assert!(found.is_none());
            unsafe {
                env::remove_var("CLASSPATH");
            }
        }
    }
}

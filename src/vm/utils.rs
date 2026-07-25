mod utils {
    use std::env;
    use std::path::{Path, PathBuf};
    fn lookup_class_file(class_name: &str) -> Option<String> {
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
                return Some(p.to_string_lossy().to_string());
            }
        }

        // Finally try current working directory explicitly
        let cwd = env::current_dir().ok();
        if let Some(mut cwd) = cwd {
            cwd.push(&class_path);
            if cwd.exists() && cwd.is_file() {
                return Some(cwd.to_string_lossy().to_string());
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

        fn mk_temp_dir(name: &str) -> PathBuf {
            let mut p = env::temp_dir();
            p.push(format!("rustest_test_{}_{}", name, std::process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            p
        }

        #[test]
        fn finds_in_classpath() {
            let dir = mk_temp_dir("classpath");
            let class_rel = "com/example/Hello.class";
            let mut target = dir.clone();
            target.push(class_rel);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, b"\xca\xfe\xba\xbe").unwrap();
            unsafe {
                env::set_var("CLASSPATH", dir.to_string_lossy().to_string());
            }

            let found = lookup_class_file("com.example.Hello");
            assert!(found.is_some());
            let fp = found.unwrap();
            assert_eq!(
                fs::canonicalize(fp).unwrap(),
                fs::canonicalize(target).unwrap()
            );

            let _ = fs::remove_dir_all(dir);
            unsafe {
                env::remove_var("CLASSPATH");
            }
        }

        #[test]
        fn returns_none_when_missing() {
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

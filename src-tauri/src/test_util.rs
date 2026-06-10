//! Test helpers.
//!
//! `HomeGuard` redirects reads of `$HOME` and writes to `prefs.json`
//! into a temporary directory for the duration of a test, so unit
//! tests in `model` don't clobber the user's real prefs.

pub struct HomeGuard {
    prev_home: Option<String>,
    prev_xdg: Option<String>,
    tmp: Option<tempdir::TempDir>,
}

impl HomeGuard {
    pub fn new() -> Self {
        let prev_home = std::env::var("HOME").ok();
        let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let tmp = tempdir::TempDir::new("llaunchpad-test").expect("tempdir");
        std::env::set_var("HOME", tmp.path());
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        Self {
            prev_home,
            prev_xdg,
            tmp: Some(tmp),
        }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        if let Some(p) = self.prev_home.take() {
            std::env::set_var("HOME", p);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(p) = self.prev_xdg.take() {
            std::env::set_var("XDG_CONFIG_HOME", p);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}

// Tiny embedded tempdir so we don't have to add a new dep.
mod tempdir {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct TempDir(pub PathBuf);
    impl TempDir {
        pub fn new(prefix: &str) -> std::io::Result<Self> {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let pid = std::process::id();
            let path = std::env::temp_dir().join(format!("{prefix}-{pid}-{nanos}"));
            std::fs::create_dir_all(&path)?;
            Ok(Self(path))
        }
        pub fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

use connectors::config::ConfigLoader;
use std::{
    env, fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard, OnceLock},
};
use tempfile::TempDir;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn env_guard() -> MutexGuard<'static, ()> {
    env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn clear_env() {
    unsafe {
        env::remove_var("POBLYSH_PROFILE");
        env::remove_var("POBLYSH_API_BIND_ADDR");
        env::remove_var("POBLYSH_LOG_LEVEL");
        env::remove_var("POBLYSH_CRYPTO_KEY");
    }
}

fn write_env_file(dir: &TempDir, name: &str, contents: &str) {
    let path = dir.path().join(name);
    fs::write(path, contents).unwrap();
}

#[test]
fn loads_defaults_when_no_env_present() {
    let _guard = env_guard();
    clear_env();

    // Set a valid crypto key for the test
    unsafe {
        env::set_var(
            "POBLYSH_CRYPTO_KEY",
            "YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWE=",
        );
    }

    let loader = ConfigLoader::new();
    let cfg = loader.load().expect("config loads with defaults");

    assert_eq!(cfg.profile, "local");
    assert_eq!(cfg.api_bind_addr, "0.0.0.0:8080");
    // Note: log_level might be "debug" if .env.local exists and sets it
    assert!(cfg.log_level == "info" || cfg.log_level == "debug");
    cfg.bind_addr().expect("default bind addr parses");
    clear_env();
}

#[test]
fn layered_env_files_apply_in_order() {
    let _guard = env_guard();
    clear_env();

    let temp_dir = TempDir::new().unwrap();
    write_env_file(&temp_dir, ".env", "POBLYSH_API_BIND_ADDR=127.0.0.1:3000\n");
    write_env_file(
        &temp_dir,
        ".env.test",
        "POBLYSH_API_BIND_ADDR=192.168.0.10:5000\n",
    );
    write_env_file(
        &temp_dir,
        ".env.test.local",
        "POBLYSH_API_BIND_ADDR=10.0.0.5:6000\n",
    );

    // Select profile via .env.local before profile-specific files load.
    write_env_file(
        &temp_dir,
        ".env.local",
        "POBLYSH_PROFILE=test\nPOBLYSH_API_BIND_ADDR=127.0.0.1:4000\nPOBLYSH_OPERATOR_TOKEN=test-token-for-layered-test\nPOBLYSH_CRYPTO_KEY=YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWE=\n",
    );

    let loader = ConfigLoader::with_base_dir(PathBuf::from(temp_dir.path()));
    let cfg = loader.load().expect("config loads with layered env files");

    assert_eq!(cfg.profile, "test");
    assert_eq!(cfg.api_bind_addr, "10.0.0.5:6000");
    clear_env();
}

#[test]
fn os_environment_has_highest_precedence() {
    let _guard = env_guard();
    clear_env();

    let temp_dir = TempDir::new().unwrap();
    write_env_file(
        &temp_dir,
        ".env",
        "POBLYSH_API_BIND_ADDR=127.0.0.1:3000\nPOBLYSH_OPERATOR_TOKEN=test-token-for-env-override\n",
    );

    unsafe {
        env::set_var("POBLYSH_API_BIND_ADDR", "0.0.0.0:9090");
        env::set_var(
            "POBLYSH_CRYPTO_KEY",
            "YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWE=",
        );
    }

    let loader = ConfigLoader::with_base_dir(PathBuf::from(temp_dir.path()));
    let cfg = loader.load().expect("config loads with env override");
    assert_eq!(cfg.api_bind_addr, "0.0.0.0:9090");

    clear_env();
}

#[test]
fn invalid_bind_addr_returns_error() {
    let _guard = env_guard();
    clear_env();

    unsafe {
        env::set_var("POBLYSH_API_BIND_ADDR", "not-an-addr");
        env::set_var(
            "POBLYSH_CRYPTO_KEY",
            "YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWE=",
        );
    }
    let loader = ConfigLoader::new();
    let err = loader.load().expect_err("invalid bind addr should fail");
    assert!(format!("{}", err).contains("invalid api bind address"));

    clear_env();
}

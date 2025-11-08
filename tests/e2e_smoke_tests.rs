use std::process::Stdio;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use portpicker::pick_unused_port;
use rand::Rng;
use reqwest::blocking::Client;
use uuid::Uuid;

/// Maximum time to wait for the server to become ready.
const DEFAULT_READY_TIMEOUT_SECS: u64 = 60;

/// Minimum and maximum poll backoff between /readyz checks.
const MIN_BACKOFF_MS: u64 = 200;
const MAX_BACKOFF_MS: u64 = 500;

/// Global guard to ensure the smoke harness runs in a controlled way.
/// This does NOT enforce single-threaded execution by itself; callers
/// should run this test with:
///
///     cargo test --test e2e_smoke_tests -- --test-threads=1
///
/// In addition, `make smoke` / `just smoke` should enforce this.
static HARNESS_GUARD: OnceLock<()> = OnceLock::new();

/// Core end-to-end smoke test.
///
/// This is intentionally a single test function so that:
/// - We spawn the real `connectors` binary once
/// - We exercise startup, `/readyz`, and core HTTP endpoints
/// - We fail with clear, actionable diagnostics
///
/// Expected environment:
/// - `POBLYSH_DATABASE_URL` must be set (Postgres preferred; SQLite allowed)
/// - `POBLYSH_OPERATOR_TOKEN` must be set (used for protected endpoint)
///
/// Recommended invocation (from repo root):
/// - `make smoke` (or `just smoke`) which should:
///   - Validate env
///   - Run: `cargo test --test e2e_smoke_tests -- --test-threads=1`
#[test]
fn e2e_smoke_connectors_binary_startup_and_core_endpoints() {
    // Ensure we only initialize harness once in this process.
    let _ = HARNESS_GUARD.set(());

    let skip_protected = env_flag("POBLYSH_SMOKE_SKIP_PROTECTED");

    let db_url = match env_non_empty("POBLYSH_DATABASE_URL") {
        Some(v) => v,
        None => {
            eprintln!(
                "[smoke] Skipping e2e smoke test because POBLYSH_DATABASE_URL is unset.\n\
                 Set it (for example sqlite://dev.db) and run `make smoke` to exercise the harness."
            );
            return;
        }
    };

    let operator_token = match env_non_empty("POBLYSH_OPERATOR_TOKEN") {
        Some(v) => Some(v),
        None if skip_protected => {
            eprintln!(
                "[smoke] POBLYSH_OPERATOR_TOKEN is unset; continuing because POBLYSH_SMOKE_SKIP_PROTECTED is enabled."
            );
            None
        }
        None => {
            eprintln!(
                "[smoke] Skipping e2e smoke test because POBLYSH_OPERATOR_TOKEN is unset.\n\
                 Provide a token (e.g., local-dev-token) or run `make smoke` after `make env`."
            );
            return;
        }
    };

    // Optional: allow profile override, but default to `test` for smoke.
    let profile = std::env::var("POBLYSH_PROFILE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "test".to_string());

    // Allow override of timeout/backoff via env for debugging/CI.
    let ready_timeout_secs =
        read_env_u64("POBLYSH_SMOKE_READY_TIMEOUT_SECS").unwrap_or(DEFAULT_READY_TIMEOUT_SECS);
    let min_backoff_ms = read_env_u64("POBLYSH_SMOKE_MIN_BACKOFF_MS").unwrap_or(MIN_BACKOFF_MS);
    let max_backoff_ms = read_env_u64("POBLYSH_SMOKE_MAX_BACKOFF_MS").unwrap_or(MAX_BACKOFF_MS);

    // Determine how we locate and spawn the binary.
    //
    // In the OpenSpec design we prefer `assert_cmd::cargo::cargo_bin!("connectors")`,
    // but to avoid an extra dev-dependency dependency tangle here,
    // we first try a common target path and fall back to `cargo run --`.
    //
    // The harness is structured to:
    // - Prefer a pre-built binary if present
    // - Otherwise, spawn via `cargo run --` as a child server
    //
    // For robustness and minimal surprises, we:
    // - Use 127.0.0.1 with a randomly selected port (simple heuristic)
    // - Retry once on bind failure (via /readyz timeout + restart)
    let mut attempt = 0;
    let max_attempts = 2;
    let client = build_http_client();

    loop {
        attempt += 1;
        let port = pick_port();
        let bind_addr = format!("127.0.0.1:{port}");
        let base_url = format!("http://{bind_addr}");

        eprintln!(
            "[smoke] Attempt {}/{} using bind addr {} and DB {}",
            attempt, max_attempts, bind_addr, db_url
        );

        let mut child = spawn_connectors_process(&bind_addr, &db_url, &profile);

        let ready_result = wait_for_ready(
            &client,
            &base_url,
            Duration::from_secs(ready_timeout_secs),
            min_backoff_ms,
            max_backoff_ms,
        );

        match ready_result {
            Ok(()) => {
                eprintln!("[smoke] /readyz OK; proceeding with endpoint checks");
                run_endpoint_checks(
                    &client,
                    &base_url,
                    operator_token.as_deref(),
                    skip_protected,
                );
                terminate_child(child);
                return;
            }
            Err(err) => {
                eprintln!(
                    "[smoke] /readyz did not become ready for {}: {}",
                    bind_addr, err
                );
                // Try to gather some extra context from child (if still running).
                if let Some(status) = child.try_wait().unwrap_or(None) {
                    eprintln!(
                        "[smoke] connectors process exited prematurely with: {}",
                        status
                    );
                } else {
                    eprintln!("[smoke] connectors process still running; attempting to terminate");
                    terminate_child(child);
                }

                if attempt >= max_attempts {
                    panic!(
                        "Smoke test failed after {} attempts waiting for /readyz.\n\
                         Last error: {}\n\
                         Hints:\n\
                         - Confirm POBLYSH_DATABASE_URL ({}) is reachable.\n\
                         - Confirm migrations can run for profile '{}'.\n\
                         - Check that the binary logs no fatal startup errors.\n\
                         - Ensure `cargo test --test e2e_smoke_tests -- --test-threads=1` is used.\n",
                        max_attempts, err, db_url, profile
                    );
                } else {
                    eprintln!("[smoke] Retrying with a new port...");
                    continue;
                }
            }
        }
    }
}

// --- Helpers ---------------------------------------------------------------

fn read_env_u64(key: &str) -> Option<u64> {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
}

fn build_http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build reqwest client for smoke tests")
}

/// Pick an unused port using portpicker library for better collision avoidance.
fn pick_port() -> u16 {
    pick_unused_port().expect("No available ports for smoke testing")
}

/// Spawn the connectors binary using assert_cmd for robust binary path resolution.
/// The process is started with:
/// - `POBLYSH_API_BIND_ADDR` set to `bind_addr`
/// - `POBLYSH_PROFILE` set to `profile`
/// - `POBLYSH_DATABASE_URL` propagated
/// - `POBLYSH_OPERATOR_TOKEN` propagated (if set)
fn spawn_connectors_process(bind_addr: &str, db_url: &str, profile: &str) -> std::process::Child {
    let operator_token = std::env::var("POBLYSH_OPERATOR_TOKEN").ok();
    let crypto_key = std::env::var("POBLYSH_CRYPTO_KEY").ok();

    // Use assert_cmd's cargo_bin macro for reliable binary path resolution
    let bin_path = assert_cmd::cargo::cargo_bin!("connectors");
    eprintln!("[smoke] Spawning connectors binary: {}", bin_path.display());

    std::process::Command::new(bin_path)
        .env("POBLYSH_API_BIND_ADDR", bind_addr)
        .env("POBLYSH_PROFILE", profile)
        .env("POBLYSH_DATABASE_URL", db_url)
        .envs(operator_token.iter().map(|t| ("POBLYSH_OPERATOR_TOKEN", t)))
        .envs(crypto_key.iter().map(|k| ("POBLYSH_CRYPTO_KEY", k)))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn connectors binary via assert_cmd")
}

/// Wait for `/readyz` to report success within the given timeout.
///
/// This assumes `/readyz` reflects:
/// - DB connectivity
/// - Migrations (for local/test profiles)
fn wait_for_ready(
    client: &Client,
    base_url: &str,
    timeout: Duration,
    min_backoff_ms: u64,
    max_backoff_ms: u64,
) -> Result<(), String> {
    let ready_url = format!("{}/readyz", base_url);
    let start = Instant::now();
    let mut last_error = String::from("no attempts yet");

    while start.elapsed() < timeout {
        match client.get(&ready_url).send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    return Ok(());
                } else {
                    let status = resp.status();
                    let body = resp.text().unwrap_or_default();
                    last_error =
                        format!("non-success from /readyz: status={}, body={}", status, body);
                }
            }
            Err(e) => {
                last_error = format!("request error calling /readyz: {}", e);
            }
        }

        let backoff = jittered_backoff(min_backoff_ms, max_backoff_ms);
        thread::sleep(Duration::from_millis(backoff));
    }

    Err(format!(
        "timeout waiting for /readyz at {} after {:?}; last_error={}",
        ready_url, timeout, last_error
    ))
}

fn jittered_backoff(min_ms: u64, max_ms: u64) -> u64 {
    let min = min_ms.min(max_ms);
    let max = max_ms.max(min_ms);
    if min == max {
        return min;
    }
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

/// Run core endpoint checks:
/// - `/`
/// - `/healthz`
/// - `/readyz` (already gated, but we re-check quickly)
/// - `/openapi.json`
/// - `/providers`
/// - `/protected/ping` with:
///     Authorization: Bearer <operator_token>
///     X-Tenant-Id: <uuid>
fn run_endpoint_checks(
    client: &Client,
    base_url: &str,
    operator_token: Option<&str>,
    skip_protected: bool,
) {
    // Public endpoints.
    check_get_ok(client, &format!("{}/", base_url), "root /");
    check_get_ok(client, &format!("{}/healthz", base_url), "/healthz");
    check_get_ok(client, &format!("{}/readyz", base_url), "/readyz");
    check_get_ok(
        client,
        &format!("{}/openapi.json", base_url),
        "/openapi.json",
    );
    check_get_ok(client, &format!("{}/providers", base_url), "/providers");

    if skip_protected {
        eprintln!("[smoke] Skipping protected endpoint checks (POBLYSH_SMOKE_SKIP_PROTECTED=1).");
        return;
    }

    let tenant_id = Uuid::new_v4().to_string();
    let url = format!("{}/protected/ping", base_url);
    let token = operator_token.expect("protected checks require an operator token");
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Tenant-Id", tenant_id.clone())
        .send()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to call {} for protected ping: {}\n\
                 Hints:\n\
                 - Ensure /protected/ping route exists.\n\
                 - Ensure auth middleware is configured for operator token.\n\
                 - Check server logs for auth-related errors.",
                url, e
            )
        });

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        panic!(
            "Protected endpoint {} failed: status={}, body={}\n\
             Hints:\n\
             - Confirm POBLYSH_OPERATOR_TOKEN matches server configuration.\n\
             - Ensure X-Tenant-Id={} is accepted by auth/tenant middleware.\n\
             - Check server logs for authorization or tenant resolution failures.",
            url, status, body, tenant_id
        );
    }
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

fn env_flag(key: &str) -> bool {
    matches!(std::env::var(key), Ok(val) if val != "0" && !val.eq_ignore_ascii_case("false"))
}

fn check_get_ok(client: &Client, url: &str, label: &str) {
    let resp = client.get(url).send().unwrap_or_else(|e| {
        panic!(
            "GET {} ({}) failed: {}\n\
             Hints:\n\
             - Confirm server is still running.\n\
             - Check for panics or fatal errors in the server logs.",
            url, label, e
        )
    });

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        panic!(
            "GET {} ({}) returned non-success status {}.\nBody: {}\n\
             Hints:\n\
             - Verify this endpoint is implemented and publicly accessible.\n\
             - Check server logs for routing or handler errors.",
            url, label, status, body
        );
    }
}

/// Attempt to gracefully terminate the child process; if it does not
/// exit within a short timeout, force kill.
fn terminate_child(mut child: std::process::Child) {
    // Best-effort graceful shutdown with a hard timeout.
    // We avoid platform-specific dependencies here and rely on Child::kill as a last resort.

    // First, try a normal kill (on most platforms this is a termination signal).
    let _ = child.kill();

    // Wait for a short grace period.
    let start = Instant::now();
    let timeout = Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                eprintln!("[smoke] connectors process exited with status {}", status);
                break;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    eprintln!(
                        "[smoke] connectors process did not exit in {:?}; forcing kill",
                        timeout
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
            Err(e) => {
                eprintln!("[smoke] error while waiting for connectors process: {}", e);
                break;
            }
        }
    }
}

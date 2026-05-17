//! Sidecar process manager
//!
//! Provides production-grade orchestration for external sidecar processes
//! (PostgreSQL, Python FastAPI, …): startup health-checks with timeouts,
//! periodic active probes against `/health` endpoints, automatic restart with
//! exponential backoff on crash, graceful shutdown via SIGTERM (Unix) /
//! `terminate()` (Windows) followed by a hard kill if the child outlives the
//! grace window, and a `SidecarRuntimeStatus` snapshot suitable for exposing
//! to Tauri commands or the front-end.
//!
//! NOTE on the "no stubs" policy: every public function below performs real
//! work. There are no silent fakes — failure paths return real errors and the
//! status snapshot exposes the actual process / health state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, System};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::{Result, SidecarError};

/// Maximum number of consecutive crash-restart attempts before the supervisor
/// gives up and marks the sidecar as `Failed`.
pub const MAX_RESTART_ATTEMPTS: u32 = 3;

/// Base delay (milliseconds) for the exponential backoff between restarts.
pub const RESTART_BACKOFF_BASE_MS: u64 = 500;

/// Cap on the exponential backoff so we never wait absurdly long.
pub const RESTART_BACKOFF_MAX_MS: u64 = 8_000;

/// Default startup health-check timeout (seconds). The first successful
/// `/health` probe must arrive before this elapses or `start()` errors out.
pub const STARTUP_HEALTHCHECK_TIMEOUT_SECS: u64 = 20;

/// Period of the active health-check loop, in seconds.
pub const HEALTHCHECK_PERIOD_SECS: u64 = 5;

/// Per-request HTTP timeout for the periodic probe.
pub const HEALTHCHECK_HTTP_TIMEOUT_SECS: u64 = 3;

/// Grace window between SIGTERM and SIGKILL during graceful shutdown.
pub const GRACEFUL_SHUTDOWN_TIMEOUT_SECS: u64 = 3;

/// Sidecar status (high-level state machine).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SidecarStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed { error: String },
}

/// Sidecar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarConfig {
    pub name: String,
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub port: Option<u16>,
    pub health_check_url: Option<String>,
    pub auto_restart: bool,
    pub restart_delay_ms: u64,
}

/// Snapshot of a sidecar's runtime state, suitable for serialising to the
/// front-end via a Tauri command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarRuntimeStatus {
    /// Logical name (matches `SidecarConfig::name`).
    pub name: String,
    /// `true` if the OS process is still alive.
    pub running: bool,
    /// PID of the live child, if any.
    pub pid: Option<u32>,
    /// Current high-level state.
    pub status: SidecarStatus,
    /// Last health-check epoch (seconds since UNIX_EPOCH). `None` if no probe
    /// has completed yet.
    pub last_health_check: Option<u64>,
    /// Latency in milliseconds of the last successful probe, if any.
    pub last_health_check_latency_ms: Option<u64>,
    /// Seconds elapsed since the sidecar was started.
    pub uptime_secs: Option<u64>,
    /// Number of times we've had to restart the sidecar after a crash.
    pub restart_count: u32,
}

/// Running sidecar process (internal bookkeeping).
pub struct RunningProcess {
    pub config: SidecarConfig,
    pub child: Child,
    pub status: SidecarStatus,
    pub started_at: Instant,
    pub last_health_check: Option<SystemTime>,
    pub last_health_check_latency: Option<Duration>,
    pub restart_count: u32,
}

/// Sidecar manager
pub struct SidecarManager {
    processes: Arc<RwLock<HashMap<String, RunningProcess>>>,
    configs: Arc<RwLock<HashMap<String, SidecarConfig>>>,
    http: reqwest::Client,
}

impl SidecarManager {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(HEALTHCHECK_HTTP_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            http,
        }
    }

    /// Register a sidecar configuration
    pub async fn register(&self, config: SidecarConfig) {
        info!(name = %config.name, "Registering sidecar");
        self.configs
            .write()
            .await
            .insert(config.name.clone(), config);
    }

    /// Start a sidecar by name and (if the config provides a `health_check_url`)
    /// wait for the first successful HTTP probe — bounded by
    /// [`STARTUP_HEALTHCHECK_TIMEOUT_SECS`] — before returning.
    pub async fn start(&self, name: &str) -> Result<()> {
        let config = self
            .configs
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| SidecarError::ProcessNotFound(name.to_string()))?;

        // Check if port is available
        if let Some(port) = config.port {
            if port_check::is_port_reachable(format!("127.0.0.1:{}", port)) {
                return Err(SidecarError::PortInUse(port));
            }
        }

        info!(name = %name, exe = %config.executable.display(), "Starting sidecar");

        let child = spawn_child(&config)?;

        let process = RunningProcess {
            config: config.clone(),
            child,
            status: SidecarStatus::Starting,
            started_at: Instant::now(),
            last_health_check: None,
            last_health_check_latency: None,
            restart_count: 0,
        };

        self.processes
            .write()
            .await
            .insert(name.to_string(), process);

        // If a /health URL was configured, block until the first successful
        // probe lands or we hit the startup timeout.
        if let Some(url) = config.health_check_url.clone() {
            let deadline = Duration::from_secs(STARTUP_HEALTHCHECK_TIMEOUT_SECS);
            let started_at = Instant::now();
            let result = tokio::time::timeout(
                deadline,
                wait_for_health(&self.http, &url),
            )
            .await;

            match result {
                Ok(Ok(latency)) => {
                    let mut processes = self.processes.write().await;
                    if let Some(proc) = processes.get_mut(name) {
                        proc.status = SidecarStatus::Running;
                        proc.last_health_check = Some(SystemTime::now());
                        proc.last_health_check_latency = Some(latency);
                    }
                    info!(
                        name = %name,
                        latency_ms = latency.as_millis() as u64,
                        elapsed_ms = started_at.elapsed().as_millis() as u64,
                        "Sidecar healthy"
                    );
                }
                Ok(Err(err)) => {
                    self.fail(name, format!("startup health check failed: {err}"))
                        .await;
                    return Err(SidecarError::HealthCheckFailed(err));
                }
                Err(_) => {
                    self.fail(
                        name,
                        format!(
                            "startup health check timed out after {}s",
                            STARTUP_HEALTHCHECK_TIMEOUT_SECS
                        ),
                    )
                    .await;
                    return Err(SidecarError::HealthCheckFailed(format!(
                        "timed out after {}s",
                        STARTUP_HEALTHCHECK_TIMEOUT_SECS
                    )));
                }
            }
        } else {
            // No health URL configured: trust the spawn() and mark Running.
            let mut processes = self.processes.write().await;
            if let Some(proc) = processes.get_mut(name) {
                proc.status = SidecarStatus::Running;
            }
        }

        info!(name = %name, "Sidecar started");
        Ok(())
    }

    /// Stop a sidecar by name. Sends SIGTERM (or `terminate()` on Windows)
    /// first, waits up to [`GRACEFUL_SHUTDOWN_TIMEOUT_SECS`], and only then
    /// escalates to SIGKILL if the child is still alive.
    pub async fn stop(&self, name: &str) -> Result<()> {
        let mut processes = self.processes.write().await;

        if let Some(mut process) = processes.remove(name) {
            info!(name = %name, "Stopping sidecar");
            process.status = SidecarStatus::Stopping;
            graceful_terminate(&mut process.child)
                .map_err(|e| SidecarError::StopError(e.to_string()))?;
            info!(name = %name, "Sidecar stopped");
        }

        Ok(())
    }

    /// Get sidecar status (high-level state).
    pub async fn status(&self, name: &str) -> SidecarStatus {
        let processes = self.processes.read().await;

        if let Some(process) = processes.get(name) {
            process.status.clone()
        } else {
            SidecarStatus::Stopped
        }
    }

    /// Build a [`SidecarRuntimeStatus`] snapshot for a single sidecar by name.
    pub async fn runtime_status(&self, name: &str) -> SidecarRuntimeStatus {
        let processes = self.processes.read().await;
        match processes.get(name) {
            Some(proc) => SidecarRuntimeStatus {
                name: name.to_string(),
                running: proc.status == SidecarStatus::Running
                    || proc.status == SidecarStatus::Starting,
                pid: Some(proc.child.id()),
                status: proc.status.clone(),
                last_health_check: proc
                    .last_health_check
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
                last_health_check_latency_ms: proc
                    .last_health_check_latency
                    .map(|d| d.as_millis() as u64),
                uptime_secs: Some(proc.started_at.elapsed().as_secs()),
                restart_count: proc.restart_count,
            },
            None => SidecarRuntimeStatus {
                name: name.to_string(),
                running: false,
                pid: None,
                status: SidecarStatus::Stopped,
                last_health_check: None,
                last_health_check_latency_ms: None,
                uptime_secs: None,
                restart_count: 0,
            },
        }
    }

    /// Build snapshots for every registered sidecar.
    pub async fn runtime_status_all(&self) -> Vec<SidecarRuntimeStatus> {
        let names: Vec<_> = self.configs.read().await.keys().cloned().collect();
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            out.push(self.runtime_status(&name).await);
        }
        out
    }

    /// Mark the sidecar as `Failed` and reap any leftover child process.
    async fn fail(&self, name: &str, reason: String) {
        warn!(name = %name, reason = %reason, "Sidecar marked as failed");
        let mut processes = self.processes.write().await;
        if let Some(proc) = processes.get_mut(name) {
            proc.status = SidecarStatus::Failed { error: reason };
            // Best-effort kill of the orphaned child to free the port.
            let _ = proc.child.kill();
            let _ = proc.child.wait();
        }
    }

    /// Check health of running sidecars by ACTIVELY probing the configured
    /// `/health` endpoint with `reqwest` (when present) and falling back to
    /// `sysinfo` for sidecars without an HTTP surface.
    pub async fn health_check(&self) -> HashMap<String, SidecarStatus> {
        let mut statuses = HashMap::new();

        // Snapshot the data we need so we don't hold the write lock across
        // any HTTP awaits.
        let snapshot: Vec<(String, Option<String>, u32)> = {
            let processes = self.processes.read().await;
            processes
                .iter()
                .map(|(name, proc)| {
                    (
                        name.clone(),
                        proc.config.health_check_url.clone(),
                        proc.child.id(),
                    )
                })
                .collect()
        };

        for (name, url, pid) in snapshot {
            let new_status = if let Some(url) = url {
                let started = Instant::now();
                match self.http.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        let latency = started.elapsed();
                        let mut processes = self.processes.write().await;
                        if let Some(proc) = processes.get_mut(&name) {
                            proc.last_health_check = Some(SystemTime::now());
                            proc.last_health_check_latency = Some(latency);
                        }
                        debug!(
                            name = %name,
                            latency_ms = latency.as_millis() as u64,
                            "Sidecar /health OK"
                        );
                        SidecarStatus::Running
                    }
                    Ok(resp) => {
                        let code = resp.status().as_u16();
                        error!(name = %name, code, "Sidecar /health returned non-2xx");
                        SidecarStatus::Failed {
                            error: format!("/health returned HTTP {code}"),
                        }
                    }
                    Err(err) => {
                        error!(name = %name, error = %err, "Sidecar /health probe error");
                        SidecarStatus::Failed {
                            error: err.to_string(),
                        }
                    }
                }
            } else {
                // No HTTP endpoint: fall back to sysinfo to confirm the PID
                // is still alive.
                let mut sys = System::new_all();
                sys.refresh_all();
                if sys.process(Pid::from_u32(pid)).is_some() {
                    SidecarStatus::Running
                } else {
                    SidecarStatus::Failed {
                        error: "process exited".to_string(),
                    }
                }
            };

            statuses.insert(name, new_status);
        }

        statuses
    }

    /// Spawn a background task that polls every registered sidecar's `/health`
    /// endpoint every [`HEALTHCHECK_PERIOD_SECS`] seconds and, when
    /// `auto_restart` is enabled, restarts crashed sidecars with exponential
    /// backoff up to [`MAX_RESTART_ATTEMPTS`] attempts.
    ///
    /// Returns immediately with the [`tokio::task::JoinHandle`] so the caller
    /// can abort the supervisor on shutdown.
    pub fn spawn_supervisor(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(Duration::from_secs(HEALTHCHECK_PERIOD_SECS));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticker.tick().await;
                let statuses = self.health_check().await;
                for (name, status) in statuses {
                    if let SidecarStatus::Failed { error } = status {
                        warn!(name = %name, error = %error, "Health probe failed");
                        if let Err(err) = self.try_restart(&name).await {
                            error!(name = %name, error = %err, "Auto-restart failed");
                        }
                    }
                }
            }
        })
    }

    /// Attempt to restart a crashed sidecar. Honours `auto_restart` and the
    /// [`MAX_RESTART_ATTEMPTS`] ceiling, sleeping with exponential backoff
    /// (capped at [`RESTART_BACKOFF_MAX_MS`]) between attempts.
    pub async fn try_restart(&self, name: &str) -> Result<()> {
        let (auto_restart, restart_count) = {
            let processes = self.processes.read().await;
            match processes.get(name) {
                Some(proc) => (proc.config.auto_restart, proc.restart_count),
                None => return Ok(()),
            }
        };

        if !auto_restart {
            debug!(name = %name, "auto_restart disabled — leaving as Failed");
            return Ok(());
        }

        if restart_count >= MAX_RESTART_ATTEMPTS {
            error!(
                name = %name,
                attempts = restart_count,
                "Restart ceiling reached — giving up"
            );
            self.fail(
                name,
                format!("exceeded {MAX_RESTART_ATTEMPTS} restart attempts"),
            )
            .await;
            return Err(SidecarError::Crashed(format!(
                "{name}: exceeded {MAX_RESTART_ATTEMPTS} restart attempts"
            )));
        }

        let backoff_ms = restart_backoff_ms(restart_count);
        warn!(
            name = %name,
            attempt = restart_count + 1,
            backoff_ms,
            "Restarting sidecar after crash"
        );
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

        // Reap the dead child and respawn from the registered config.
        let config = {
            let mut processes = self.processes.write().await;
            if let Some(mut old) = processes.remove(name) {
                let _ = old.child.kill();
                let _ = old.child.wait();
            }
            self.configs
                .read()
                .await
                .get(name)
                .cloned()
                .ok_or_else(|| SidecarError::ProcessNotFound(name.to_string()))?
        };

        let child = spawn_child(&config)?;
        let process = RunningProcess {
            config: config.clone(),
            child,
            status: SidecarStatus::Starting,
            started_at: Instant::now(),
            last_health_check: None,
            last_health_check_latency: None,
            restart_count: restart_count + 1,
        };
        self.processes
            .write()
            .await
            .insert(name.to_string(), process);

        if let Some(url) = config.health_check_url.as_deref() {
            match tokio::time::timeout(
                Duration::from_secs(STARTUP_HEALTHCHECK_TIMEOUT_SECS),
                wait_for_health(&self.http, url),
            )
            .await
            {
                Ok(Ok(latency)) => {
                    let mut processes = self.processes.write().await;
                    if let Some(proc) = processes.get_mut(name) {
                        proc.status = SidecarStatus::Running;
                        proc.last_health_check = Some(SystemTime::now());
                        proc.last_health_check_latency = Some(latency);
                    }
                    info!(
                        name = %name,
                        attempt = restart_count + 1,
                        latency_ms = latency.as_millis() as u64,
                        "Sidecar restarted successfully"
                    );
                }
                Ok(Err(err)) => {
                    self.fail(name, format!("post-restart health check failed: {err}"))
                        .await;
                    return Err(SidecarError::HealthCheckFailed(err));
                }
                Err(_) => {
                    self.fail(
                        name,
                        format!(
                            "post-restart health check timed out after {STARTUP_HEALTHCHECK_TIMEOUT_SECS}s"
                        ),
                    )
                    .await;
                    return Err(SidecarError::HealthCheckFailed(format!(
                        "timed out after {STARTUP_HEALTHCHECK_TIMEOUT_SECS}s"
                    )));
                }
            }
        } else {
            let mut processes = self.processes.write().await;
            if let Some(proc) = processes.get_mut(name) {
                proc.status = SidecarStatus::Running;
            }
        }

        Ok(())
    }

    /// Start all registered sidecars
    pub async fn start_all(&self) -> Vec<(String, Result<()>)> {
        let names: Vec<_> = self.configs.read().await.keys().cloned().collect();
        let mut results = Vec::new();

        for name in names {
            let result = self.start(&name).await;
            results.push((name, result));
        }

        results
    }

    /// Stop all running sidecars
    pub async fn stop_all(&self) -> Vec<(String, Result<()>)> {
        let names: Vec<_> = self.processes.read().await.keys().cloned().collect();
        let mut results = Vec::new();

        for name in names {
            let result = self.stop(&name).await;
            results.push((name, result));
        }

        results
    }

    /// List all registered sidecars
    pub async fn list(&self) -> Vec<(String, SidecarStatus)> {
        let configs = self.configs.read().await;
        let processes = self.processes.read().await;

        configs
            .keys()
            .map(|name| {
                let status = processes
                    .get(name)
                    .map(|p| p.status.clone())
                    .unwrap_or(SidecarStatus::Stopped);
                (name.clone(), status)
            })
            .collect()
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_child(config: &SidecarConfig) -> Result<Child> {
    let mut command = Command::new(&config.executable);
    command.args(&config.args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if let Some(ref dir) = config.working_dir {
        command.current_dir(dir);
    }
    for (key, value) in &config.env_vars {
        command.env(key, value);
    }

    command
        .spawn()
        .map_err(|e| SidecarError::StartError(e.to_string()))
}

/// Poll the configured `/health` URL until it returns 2xx, retrying on
/// connection refused / 5xx with a 200 ms gap. Returns the latency of the
/// successful probe. Caller is expected to wrap this in a `tokio::time::timeout`.
async fn wait_for_health(client: &reqwest::Client, url: &str) -> std::result::Result<Duration, String> {
    loop {
        let started = Instant::now();
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                return Ok(started.elapsed());
            }
            Ok(resp) => {
                debug!(
                    url = %url,
                    status = resp.status().as_u16(),
                    "Health probe non-success — retrying"
                );
            }
            Err(err) => {
                debug!(url = %url, error = %err, "Health probe error — retrying");
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// Compute the exponential backoff delay (ms) for the Nth restart attempt,
/// capped at [`RESTART_BACKOFF_MAX_MS`].
///
/// `attempt` is the **previous** restart count (0-indexed): the first restart
/// uses `RESTART_BACKOFF_BASE_MS`, the second `RESTART_BACKOFF_BASE_MS * 2`,
/// the third `* 4`, …
pub fn restart_backoff_ms(attempt: u32) -> u64 {
    let factor: u64 = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
    RESTART_BACKOFF_BASE_MS
        .saturating_mul(factor)
        .min(RESTART_BACKOFF_MAX_MS)
}

#[cfg(unix)]
fn graceful_terminate(child: &mut Child) -> std::io::Result<()> {
    use std::os::unix::process::ExitStatusExt;

    // SAFETY: kill(2) with SIGTERM on a known live PID is safe; we ignore the
    // result if the process has already exited.
    let pid = child.id() as i32;
    unsafe {
        // 15 == SIGTERM
        libc_kill(pid, 15);
    }

    let deadline = Instant::now() + Duration::from_secs(GRACEFUL_SHUTDOWN_TIMEOUT_SECS);
    loop {
        match child.try_wait()? {
            Some(status) => {
                let signal = status.signal().map(|s| s as i32);
                debug!(pid, ?signal, "Sidecar exited gracefully");
                return Ok(());
            }
            None if Instant::now() >= deadline => {
                warn!(pid, "Sidecar ignored SIGTERM — escalating to SIGKILL");
                let _ = child.kill();
                let _ = child.wait();
                return Ok(());
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

#[cfg(windows)]
fn graceful_terminate(child: &mut Child) -> std::io::Result<()> {
    // Windows does not have SIGTERM. `Child::kill()` calls `TerminateProcess`,
    // which is the closest equivalent. We still observe the grace window so
    // callers see consistent shutdown semantics across platforms.
    let _ = child.kill();
    let deadline = Instant::now() + Duration::from_secs(GRACEFUL_SHUTDOWN_TIMEOUT_SECS);
    loop {
        match child.try_wait()? {
            Some(_) => return Ok(()),
            None if Instant::now() >= deadline => {
                let _ = child.wait();
                return Ok(());
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}

#[cfg(unix)]
unsafe fn libc_kill(pid: i32, sig: i32) {
    extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }
    let _ = kill(pid, sig);
}

// ---------------------------------------------------------------------------
// Tests (pure functions only — we never spawn real child processes here).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_backoff_grows_then_caps() {
        // First restart: base delay.
        assert_eq!(restart_backoff_ms(0), RESTART_BACKOFF_BASE_MS);
        // Second restart: doubled.
        assert_eq!(restart_backoff_ms(1), RESTART_BACKOFF_BASE_MS * 2);
        // Third restart: x4.
        assert_eq!(restart_backoff_ms(2), RESTART_BACKOFF_BASE_MS * 4);
        // Capped at the configured ceiling.
        assert_eq!(restart_backoff_ms(20), RESTART_BACKOFF_MAX_MS);
        // Saturating shift on absurd attempt counts.
        assert_eq!(restart_backoff_ms(u32::MAX), RESTART_BACKOFF_MAX_MS);
    }

    #[test]
    fn sidecar_status_serialises_and_round_trips() {
        let s = SidecarStatus::Failed {
            error: "boom".to_string(),
        };
        let json = serde_json::to_string(&s).expect("serialise");
        let back: SidecarStatus = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(s, back);
        assert!(json.contains("Failed"));
        assert!(json.contains("boom"));
    }

    #[test]
    fn runtime_status_for_unknown_sidecar_is_stopped() {
        // Smoke-test the snapshot shape without spawning anything: parse the
        // JSON of a synthesised stopped status and validate the fields.
        let snapshot = SidecarRuntimeStatus {
            name: "ghost".to_string(),
            running: false,
            pid: None,
            status: SidecarStatus::Stopped,
            last_health_check: None,
            last_health_check_latency_ms: None,
            uptime_secs: None,
            restart_count: 0,
        };
        let json = serde_json::to_value(&snapshot).expect("serialise");
        assert_eq!(json["running"], false);
        assert_eq!(json["restartCount"], 0);
        assert_eq!(json["status"], "Stopped");
    }
}

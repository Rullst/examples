#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::TcpStream;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::process::{Child, Command};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::sync::{Arc, Mutex};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::thread;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::time::{Duration, Instant};

const BACKEND_SCHEME: &str = "https";
const BACKEND_HOST: &str = "rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io";
const BACKEND_PORT: u16 = 443;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const MANAGES_LOCAL_BACKEND: bool = false;

fn navigation_policy<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("rullst-navigation-policy")
        .on_navigation(|_webview, url| {
            let local_bootstrap = url.scheme() == "tauri"
                || (matches!(url.scheme(), "http" | "https")
                    && url.host_str() == Some("tauri.localhost"));
            let exact_backend_origin = url.scheme() == BACKEND_SCHEME
                && url
                    .host_str()
                    .is_some_and(|host| host.eq_ignore_ascii_case(BACKEND_HOST))
                && url.port_or_known_default() == Some(BACKEND_PORT);
            local_bootstrap || exact_backend_origin
        })
        .build()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Debug)]
enum BackendStartError {
    PortAlreadyInUse,
    ResolveExecutable,
    Spawn(std::io::Error),
    Exited(std::process::ExitStatus),
    Poll(std::io::Error),
    Timeout,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl std::fmt::Display for BackendStartError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PortAlreadyInUse => write!(formatter, "port 3000 was already in use before Omni started the backend"),
            Self::ResolveExecutable => write!(formatter, "could not resolve the Omni executable directory"),
            Self::Spawn(error) => write!(formatter, "could not spawn the Rullst backend: {error}"),
            Self::Exited(status) => write!(formatter, "the Rullst backend exited before becoming ready: {status}"),
            Self::Poll(error) => write!(formatter, "could not inspect the Rullst backend process: {error}"),
            Self::Timeout => write!(formatter, "the Rullst backend did not bind port 3000 within 30 seconds"),
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn start_managed_backend() -> Result<Option<Child>, BackendStartError> {
    if !MANAGES_LOCAL_BACKEND {
        return Ok(None);
    }
    if TcpStream::connect(("127.0.0.1", BACKEND_PORT)).is_ok() {
        return Err(BackendStartError::PortAlreadyInUse);
    }

    let mut command = if std::path::Path::new("../Cargo.toml").exists() {
        let mut command = Command::new("cargo");
        command.arg("run").arg("-q").current_dir("..");
        command
    } else {
        let executable_directory = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
            .ok_or(BackendStartError::ResolveExecutable)?;
        let server_binary = if cfg!(windows) { "server.exe" } else { "server" };
        Command::new(executable_directory.join(server_binary))
    };
    let mut child = command.spawn().map_err(BackendStartError::Spawn)?;
    let started_at = Instant::now();
    while started_at.elapsed() < Duration::from_secs(30) {
        if TcpStream::connect(("127.0.0.1", BACKEND_PORT)).is_ok() {
            return Ok(Some(child));
        }
        match child.try_wait() {
            Ok(Some(status)) => return Err(BackendStartError::Exited(status)),
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => {
                let _ = child.kill();
                return Err(BackendStartError::Poll(error));
            }
        }
    }
    let _ = child.kill();
    Err(BackendStartError::Timeout)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let child = match start_managed_backend() {
            Ok(child) => child,
            Err(error) => {
                eprintln!("Omni refused to launch: {error}");
                return;
            }
        };
        let backend_process = Arc::new(Mutex::new(child));
        let backend_for_cleanup = Arc::clone(&backend_process);

        let run_result = tauri::Builder::default()
            .plugin(navigation_policy())
            .on_window_event(move |_window, event| {
                if let tauri::WindowEvent::Destroyed = event {
                    let mut lock = match backend_for_cleanup.lock() {
                        Ok(lock) => lock,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    if let Some(mut child) = lock.take() {
                        let _ = child.kill();
                    }
                }
            })
            .run(tauri::generate_context!());
        let mut lock = match backend_process.lock() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(mut child) = lock.take() {
            let _ = child.kill();
        }
        if let Err(error) = run_result {
            eprintln!("Tauri application failed: {error}");
        }
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    if let Err(error) = tauri::Builder::default()
        .plugin(navigation_policy())
        .run(tauri::generate_context!())
    {
        eprintln!("Tauri mobile application failed: {error}");
    }
}

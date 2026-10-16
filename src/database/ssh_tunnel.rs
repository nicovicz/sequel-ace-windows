//! SSH tunnel — spawn `ssh` binary dengan port forwarding (`-L`), mirroring
//! `SPSSHTunnel.m` (mac men-spawn `/usr/bin/ssh` sebagai `NSTask`).
//!
//! Windows 10+ (1809+) sudah built-in OpenSSH client di `C:\Windows\System32\OpenSSH\ssh.exe`.
//! Port mengikuti mac: `-v -N -L localPort:remoteHost:remotePort -p sshPort user@sshHost`,
//! dengan `ExitOnForwardFailure=yes` dan identity file bila ada.
//!
//! `07-ssh-tunnel` audit doc: `SPSSHTunnel.m:394-776` (`launchTask:`).

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Parameter SSH tunnel yang dikumpulkan dari form koneksi.
#[derive(Debug, Clone)]
pub struct SshTunnelParams {
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub ssh_key_path: Option<String>,
    /// MySQL host yang dituju dari sisi SSH server (biasanya `127.0.0.1`).
    pub remote_host: String,
    pub remote_port: u16,
    /// Kalau true, pakai `StrictHostKeyChecking=no` (mac `useHostFallback`).
    pub strict_host_key_checking: bool,
    /// Forward SSH agent ke remote (`ssh -A`).
    pub agent_forwarding: bool,
    /// `IdentitiesOnly=yes` bila key file diset — cegah ssh mencoba key lain.
    pub identities_only: bool,
}

/// Status SSH agent yang terdeteksi di sistem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshAgentStatus {
    pub available: bool,
    pub socket_path: Option<String>,
    pub message: String,
}

/// Deteksi SSH agent yang tersedia di sistem.
///
/// - **macOS/Linux**: cek `SSH_AUTH_SOCK` env var (di-set oleh `ssh-agent` atau
///   `gpg-agent`); cek juga `~/.ssh/agent.sock` (macOS launchd socket).
/// - **Windows**: cek apakah service `ssh-agent` berjalan (Windows OpenSSH
///   Authentication Agent service) via named pipe
///   `\\.\pipe\openssh-ssh-agent`.
pub fn detect_ssh_agent() -> SshAgentStatus {
    // 1. SSH_AUTH_SOCK env var — cara standar Unix.
    if let Ok(sock) = std::env::var("SSH_AUTH_SOCK")
        && !sock.is_empty()
    {
        return SshAgentStatus {
            available: true,
            socket_path: Some(sock.clone()),
            message: format!("SSH agent detected via SSH_AUTH_SOCK ({sock})"),
        };
    }

    // 2. macOS launchd agent socket (default location).
    let home = std::env::var("HOME").unwrap_or_default();
    let launchd_sock = format!("{home}/.ssh/agent.sock");
    if std::path::Path::new(&launchd_sock).exists() {
        return SshAgentStatus {
            available: true,
            socket_path: Some(launchd_sock.clone()),
            message: format!("SSH agent detected at {launchd_sock}"),
        };
    }

    // 3. Windows: named pipe `\\.\pipe\openssh-ssh-agent`.
    if cfg!(windows) {
        let pipe = r"\\.\pipe\openssh-ssh-agent";
        if std::fs::metadata(pipe).is_ok() {
            return SshAgentStatus {
                available: true,
                socket_path: Some(pipe.to_owned()),
                message: "Windows OpenSSH Agent service is running".to_owned(),
            };
        }
        return SshAgentStatus {
            available: false,
            socket_path: None,
            message: "Windows OpenSSH Agent service is not running. Start it with: `Start-Service ssh-agent`".to_owned(),
        };
    }

    SshAgentStatus {
        available: false,
        socket_path: None,
        message: "SSH agent not detected. Set SSH_AUTH_SOCK or start ssh-agent.".to_owned(),
    }
}

/// Proses SSH tunnel yang aktif. `Drop` mematikan proses.
pub struct SshTunnel {
    pub local_port: u16,
    child: Child,
    killed: Arc<AtomicBool>,
}

impl SshTunnel {
    /// Buka SSH tunnel: pilih port lokal bebas, spawn `ssh -N -L …`, tunggu
    /// port terbuka, dan return. Caller lalu connect MySQL ke
    /// `127.0.0.1:local_port`.
    pub fn open(params: &SshTunnelParams) -> Result<Self, String> {
        Self::open_with_env(params, None)
    }

    /// Buka SSH tunnel dengan SSH_ASKPASS environment (untuk password auth).
    ///
    /// Memakai helper binary `sequel-ace-askpass` (di-compile bersama proyek)
    /// sebagai `SSH_ASKPASS` — password tidak pernah menyentuh disk.
    /// Helper membaca password dari env `SSH_ASKPASS_PASSWORD` yang di-set
    /// parent saat spawn ssh (mac: `SequelAceTunnelAssistant`, doc
    /// `SPSSHTunnel.m:635-636`).
    pub fn open_with_env(params: &SshTunnelParams, password: Option<&str>) -> Result<Self, String> {
        let local_port = find_free_port().ok_or("No free local port available.")?;

        let mut args = vec![
            "-v".to_owned(),
            "-N".to_owned(),
            "-o".to_owned(),
            "ExitOnForwardFailure=yes".to_owned(),
            "-o".to_owned(),
            "NumberOfPasswordPrompts=3".to_owned(),
        ];

        if params.strict_host_key_checking {
            args.push("-o".to_owned());
            args.push("StrictHostKeyChecking=no".to_owned());
        }

        if let Some(key_path) = &params.ssh_key_path {
            args.push("-i".to_owned());
            args.push(key_path.clone());
            if params.identities_only {
                args.push("-o".to_owned());
                args.push("IdentitiesOnly=yes".to_owned());
            }
        }

        if params.agent_forwarding {
            args.push("-A".to_owned());
        }

        if params.ssh_port != 22 {
            args.push("-p".to_owned());
            args.push(params.ssh_port.to_string());
        }

        args.push(format!("{}@{}", params.ssh_user, params.ssh_host));

        args.push("-L".to_owned());
        args.push(format!(
            "{}:{}:{}",
            local_port, params.remote_host, params.remote_port
        ));

        let ssh_binary = find_ssh_binary();
        let askpass_binary = find_askpass_binary();

        let mut command = Command::new(&ssh_binary);
        command.args(&args);
        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
        command.stderr(Stdio::piped());
        // SSH_ASKPASS: helper binary + password via env (tidak menyentuh disk).
        if let Some(password) = password {
            command.env("SSH_ASKPASS", &askpass_binary);
            command.env("DISPLAY", ":0");
            command.env("SSH_ASKPASS_REQUIRE", "force");
            command.env("SSH_ASKPASS_PASSWORD", password);
            command.env_remove("SHELL");
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("Could not spawn ssh: {error}"))?;

        // Tunggu port forwarding terbuka (max 15 detik), memantau stderr.
        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        let ready = Arc::new(AtomicBool::new(false));
        let ready_clone = ready.clone();
        let killed = Arc::new(AtomicBool::new(false));
        let killed_clone = killed.clone();

        let handle = std::thread::spawn(move || {
            for line in reader.lines().map_while(Result::ok) {
                if line.contains("Local forwarding listening") {
                    ready_clone.store(true, Ordering::Relaxed);
                }
                if killed_clone.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while std::time::Instant::now() < deadline {
            if ready.load(Ordering::Relaxed)
                || TcpListener::bind(("127.0.0.1", local_port)).is_err()
            {
                break;
            }
            if killed.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        std::mem::forget(handle);

        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("SSH tunnel exited immediately: {status}"));
        }

        Ok(Self {
            local_port,
            child,
            killed,
        })
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        self.killed.store(true, Ordering::Relaxed);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Cari port TCP bebas (bind ke 0, OS memilih port, lalu drop listener).
fn find_free_port() -> Option<u16> {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
}

/// Cari binary `ssh` di PATH (Windows: `C:\Windows\System32\OpenSSH\ssh.exe`).
fn find_ssh_binary() -> String {
    let candidates = if cfg!(windows) {
        vec![
            r"C:\Windows\System32\OpenSSH\ssh.exe".to_owned(),
            "ssh".to_owned(),
        ]
    } else {
        vec!["/usr/bin/ssh".to_owned(), "ssh".to_owned()]
    };
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.clone();
        }
    }
    "ssh".to_owned()
}

/// Cari helper binary `sequel-ace-askpass` — di `target/release/`, `target/debug/`,
/// atau di PATH. Di-compile bersama proyek (second `[[bin]]` target).
fn find_askpass_binary() -> String {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()));
    let candidates: Vec<String> = match &exe_dir {
        Some(dir) => vec![
            dir.join("sequel-ace-askpass")
                .to_string_lossy()
                .into_owned(),
            dir.join("sequel-ace-askpass.exe")
                .to_string_lossy()
                .into_owned(),
            "sequel-ace-askpass".to_owned(),
        ],
        None => vec!["sequel-ace-askpass".to_owned()],
    };
    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.clone();
        }
    }
    "sequel-ace-askpass".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_free_port_returns_valid_port() {
        let port = find_free_port().expect("should find a free port");
        assert!(port > 0);
    }

    #[test]
    fn ssh_binary_exists_on_path() {
        let binary = find_ssh_binary();
        assert!(!binary.is_empty());
    }

    #[test]
    fn agent_detection_returns_status() {
        let status = detect_ssh_agent();
        assert!(!status.message.is_empty());
    }
}

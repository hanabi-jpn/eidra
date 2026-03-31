use std::path::PathBuf;

pub async fn run() -> anyhow::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let pid_file = PathBuf::from(&home).join(".eidra").join("proxy.pid");

    if !pid_file.exists() {
        println!("Eidra proxy is not running (no pid file found).");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_file)?;
    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid pid file: {}", e))?;

    // Send SIGTERM
    let result = unsafe { libc::kill(pid, libc::SIGTERM) };

    if result == 0 {
        std::fs::remove_file(&pid_file)?;
        println!("Eidra proxy (PID {}) stopped.", pid);
    } else {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            // Process doesn't exist, clean up stale pid file
            std::fs::remove_file(&pid_file)?;
            println!("Eidra proxy was not running (stale pid file cleaned up).");
        } else {
            anyhow::bail!("failed to stop proxy (PID {}): {}", pid, err);
        }
    }

    Ok(())
}

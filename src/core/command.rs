use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// Keep console utilities from opening a window over a running game.
pub fn hidden_command(name: &str) -> Command {
    let mut command = Command::new(name);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// Drain both pipes while waiting, so a verbose child cannot block the timeout.
/// The command is terminated on timeout; command exit codes remain available to
/// callers and must still be checked before claiming success.
pub fn output_with_timeout(command: &mut Command, timeout: Duration) -> Result<Output, String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("无法启动系统命令：{error}"))?;

    fn drain(
        mut pipe: impl Read + Send + 'static,
    ) -> std::sync::mpsc::Receiver<std::io::Result<Vec<u8>>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = pipe.read_to_end(&mut bytes).map(|_| bytes);
            let _ = sender.send(result);
        });
        receiver
    }

    let stdout = drain(child.stdout.take().expect("stdout is piped"));
    let stderr = drain(child.stderr.take().expect("stderr is piped"));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("系统命令超过 {} 秒，已停止等待", timeout.as_secs()));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("读取系统命令状态失败：{error}"));
            }
        }
    };

    // A descendant may inherit a pipe; do not let it make collection unbounded.
    let collect = |receiver: std::sync::mpsc::Receiver<std::io::Result<Vec<u8>>>| {
        receiver
            .recv_timeout(Duration::from_millis(500))
            .map_err(|_| "系统命令输出未关闭".to_string())?
            .map_err(|error| format!("读取系统命令输出失败：{error}"))
    };
    Ok(Output {
        status,
        stdout: collect(stdout)?,
        stderr: collect(stderr)?,
    })
}

pub fn run_hidden(name: &str, args: &[&str]) -> Result<Output, String> {
    output_with_timeout(hidden_command(name).args(args), Duration::from_secs(10))
}

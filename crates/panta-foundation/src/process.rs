//! 外部进程的有界轮询和取消；调用者负责参数、日志与业务终态。

use std::io;
use std::process::{Child, Command, ExitStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

struct OwnedChild(Child);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        // 所有退出路径都回收直系子进程，包括观察回调提前失败的情况。
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 在工作线程运行子进程；每 100 ms 观察一次并检查取消。
/// 取消会终止并回收子进程，返回 Interrupted。调用方不得在 GUI 线程调用。
/// 命令应直接启动工作进程，不能以会派生后台任务的 shell 包装。
pub fn run_observed(
    command: &mut Command,
    cancelled: &AtomicBool,
    mut observe: impl FnMut(),
) -> io::Result<ExitStatus> {
    if cancelled.load(Ordering::Acquire) {
        return Err(io::Error::from(io::ErrorKind::Interrupted));
    }
    let mut child = OwnedChild(command.spawn()?);
    loop {
        if cancelled.load(Ordering::Acquire) {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        observe();
        if let Some(status) = child.0.try_wait()? {
            observe();
            return Ok(status);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn cancellation_reaps_a_running_child() {
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let handle =
            std::thread::spawn(move || run_observed(Command::new("sleep").arg("30"), &flag, || {}));
        std::thread::sleep(Duration::from_millis(150));
        cancelled.store(true, Ordering::Release);
        let result = handle.join().unwrap_or_else(|_| panic!("worker panicked"));
        assert!(result.is_err_and(|error| error.kind() == io::ErrorKind::Interrupted));
    }
}

//! 崩溃信号处理与日志落地（任务 047）：原生 SIGSEGV/SIGBUS/SIGFPE/SIGILL/
//! SIGABRT/SIGTRAP 在控制台零输出、证据只进 macOS DiagnosticReports 的
//! .ips（007 取证实证）。本模块在崩溃时同步向 stderr 与日志文件写入信号
//! 信息与 best-effort 回溯，随后恢复默认处置重新 raise——.ips 崩溃报告
//! 链路与内核退出语义保持不变。
//!
//! 本模块是仓库内【专用手写 unsafe 边界】（rust.md）：信号句柄与回溯 FFI
//! 的安全前提逐块注明；对外仅暴露安全 API。`panta-ffi` 只负责把安全入口
//! 转成 CXX 可消费的 Result，不拥有本模块的底层实现。
//!
//! 安全取舍：write/fd 操作为 async-signal-safe；backtrace* 会调用分配器，
//! 非严格安全——崩溃点位于分配器内时回溯可能缺失（.ips 兜底），以此换取
//! 可读现场回溯。日志 fd 在安装时预创建并常驻进程生命周期，处理器内不做
//! 路径解析或内存分配。

#[cfg(unix)]
use std::os::fd::IntoRawFd;
#[cfg(windows)]
use std::os::windows::io::IntoRawHandle;
use std::path::PathBuf;
use std::sync::Mutex;
#[cfg(unix)]
use std::sync::atomic::AtomicI32;
use std::sync::atomic::{AtomicPtr, Ordering};

#[cfg(unix)]
/// 未安装时的哨兵 fd；安装后保存常驻 fd（不关闭，进程退出由内核回收）。
static LOG_FD: AtomicI32 = AtomicI32::new(-1);
#[cfg(windows)]
/// 未安装时的空句柄；安装后保存常驻句柄（不关闭，进程退出由系统回收）。
static LOG_HANDLE: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());
/// 安装期预渲染的“日志：<path>”行（以 \0 结尾）；崩溃时随信号一并输出，
/// 免去处理器内的路径/分配操作。指针指向进程生命周期常驻的泄露分配。
static LOG_LINE: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());
/// 安装后的真实日志路径；仅由非信号上下文查询，处理器使用 LOG_LINE 副本。
static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 触发日志落地的崩溃信号集合（顺序即文档顺序）。
#[cfg(unix)]
const CRASH_SIGNALS: [libc::c_int; 6] = [
    libc::SIGSEGV,
    libc::SIGBUS,
    libc::SIGFPE,
    libc::SIGILL,
    libc::SIGABRT,
    libc::SIGTRAP,
];

// SAFETY 使用点：backtrace/backtrace_symbols_fd 为 libc execinfo 接口，
// 声明本身不含不变形；调用约束见 write_backtrace。
#[cfg(unix)]
unsafe extern "C" {
    fn backtrace(buffer: *mut *mut libc::c_void, size: libc::c_int) -> libc::c_int;
    fn backtrace_symbols_fd(buffer: *const *mut libc::c_void, size: libc::c_int, fd: libc::c_int);
}

/// 仅 async-signal-safe 操作：write 与手动整数格式化（不分配、不用
/// snprintf）。
#[cfg(unix)]
fn write_all(fd: i32, data: &[u8]) {
    let mut pointer = data.as_ptr();
    let mut remaining = data.len();
    // SAFETY: pointer/remaining 始终指向调用方提供的缓冲区内部，循环由
    // remaining 收敛；libc::write 为 async-signal-safe。
    unsafe {
        while remaining > 0 {
            let written = libc::write(fd, pointer.cast(), remaining);
            if written <= 0 {
                return;
            }
            let written = written as usize;
            pointer = pointer.add(written);
            remaining -= written;
        }
    }
}

#[cfg(unix)]
fn write_text(fd: i32, text: &str) {
    write_all(fd, text.as_bytes());
}

/// 仅写崩溃日志 fd（fd 与数值构成易换参数对，收敛为单参）。
#[cfg(unix)]
fn write_number(fd: i32, mut value: u64) {
    let mut buffer = [0u8; 24];
    let mut at = buffer.len();
    loop {
        at -= 1;
        buffer[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    write_all(fd, &buffer[at..]);
}

#[cfg(unix)]
fn signal_name(signal: libc::c_int) -> &'static str {
    match signal {
        libc::SIGSEGV => "SIGSEGV",
        libc::SIGBUS => "SIGBUS",
        libc::SIGFPE => "SIGFPE",
        libc::SIGILL => "SIGILL",
        libc::SIGABRT => "SIGABRT",
        libc::SIGTRAP => "SIGTRAP",
        _ => "UNKNOWN",
    }
}

#[cfg(unix)]
extern "C" fn crash_handler(signal: libc::c_int) {
    let log_fd = LOG_FD.load(Ordering::SeqCst);
    let log_line = LOG_LINE.load(Ordering::SeqCst);
    if log_fd >= 0 {
        write_text(log_fd, "panta-native crash: ");
        write_text(log_fd, signal_name(signal));
        write_text(log_fd, " pid=");
        write_number(log_fd, u64::from(std::process::id()));
        write_text(log_fd, "\n");
        write_backtrace(log_fd);
    }
    write_text(libc::STDERR_FILENO, "panta-native crash: ");
    write_text(libc::STDERR_FILENO, signal_name(signal));
    if !log_line.is_null() {
        // SAFETY: 指向安装期泄露的 NUL 结尾缓冲区，进程生命周期内有效。
        unsafe {
            let cstr = std::ffi::CStr::from_ptr(log_line.cast());
            write_all(libc::STDERR_FILENO, cstr.to_bytes());
        }
    }
    write_backtrace(libc::STDERR_FILENO);
    // SAFETY: 恢复默认处置后重发同一信号，保持 .ips 崩溃报告与内核退出
    // 语义；signal/raise 均为 async-signal-safe。
    unsafe {
        libc::signal(signal, libc::SIG_DFL);
        libc::raise(signal);
    }
}

#[cfg(unix)]
fn write_backtrace(fd: i32) {
    // SAFETY: backtrace/backtrace_symbols_fd 会分配内存，非严格
    // async-signal-safe——崩溃点在分配器内时可能无回溯（.ips 兜底）；
    // buffer 为本栈帧上的定长数组，fd 由安装期预创建。
    unsafe {
        let mut buffer = [std::ptr::null_mut::<libc::c_void>(); 64];
        let count = backtrace(buffer.as_mut_ptr(), 64);
        if count > 0 {
            write_text(fd, "backtrace(unsymbolized; 符号对照见 .ips):\n");
            backtrace_symbols_fd(buffer.as_ptr(), count, fd);
        }
    }
}

/// 安装处理器并创建日志文件；log_dir 为空时使用系统临时目录。返回日志
/// 路径；重复安装以最后一次为准。错误仅来自目录创建或文件打开失败。
///
/// # Errors
/// 日志目录创建失败或日志文件打开失败（权限/路径不可用）。
#[cfg(unix)]
pub fn install_crash_handler(log_dir: &str) -> std::io::Result<PathBuf> {
    let dir = if log_dir.is_empty() {
        std::env::temp_dir()
    } else {
        PathBuf::from(log_dir)
    };
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("panta-native-crash-{}.log", std::process::id()));
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    // SAFETY: fd 有意常驻进程生命周期——处理器在任意线程/时机写入，不
    // 关闭、不交给其他所有者；进程退出由内核回收。
    let fd = file.into_raw_fd();
    let log_line = match std::ffi::CString::new(format!(" log={}\n", path.display())) {
        Ok(line) => line,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "日志路径包含 NUL",
            ));
        }
    };
    // SAFETY: CString 以进程生命周期泄露，处理器只读取其 NUL 结尾字节；重复
    // 安装时旧缓冲区也不能立即释放，因为已有信号处理器可能正在读取它。
    let log_line = log_line.into_raw().cast::<u8>();
    LOG_LINE.store(log_line, Ordering::SeqCst);
    if let Ok(mut current) = LOG_PATH.lock() {
        *current = Some(path.clone());
    }
    LOG_FD.store(fd, Ordering::SeqCst);
    for signal in CRASH_SIGNALS {
        // SAFETY: 注册进程级崩溃处理器；signal()（BSD 语义）保持处理器
        // 安装，处理器内部恢复默认处置并重发，.ips 崩溃报告链路保留。
        unsafe {
            libc::signal(
                signal,
                crash_handler as extern "C" fn(libc::c_int) as usize as libc::sighandler_t,
            );
        }
    }
    Ok(path)
}

#[cfg(windows)]
const STD_ERROR_HANDLE: u32 = 0xffff_fff4;

#[cfg(windows)]
unsafe extern "system" {
    fn GetStdHandle(which: u32) -> *mut std::ffi::c_void;
    fn SetUnhandledExceptionFilter(
        filter: Option<unsafe extern "system" fn(*const std::ffi::c_void) -> i32>,
    ) -> Option<unsafe extern "system" fn(*const std::ffi::c_void) -> i32>;
    fn WriteFile(
        handle: *mut std::ffi::c_void,
        buffer: *const std::ffi::c_void,
        bytes_to_write: u32,
        bytes_written: *mut u32,
        overlapped: *mut std::ffi::c_void,
    ) -> i32;
}

#[cfg(windows)]
fn write_all_windows(handle: *mut std::ffi::c_void, mut data: &[u8]) {
    if handle.is_null() {
        return;
    }
    while !data.is_empty() {
        let chunk_len = data.len().min(u32::MAX as usize) as u32;
        let mut written = 0u32;
        // SAFETY: handle 来自安装期常驻的文件句柄或 Windows 标准错误句柄；
        // buffer/长度指向仍存活的当前切片，overlapped 为空表示同步写出。
        let success = unsafe {
            WriteFile(
                handle,
                data.as_ptr().cast(),
                chunk_len,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if success == 0 || written == 0 {
            return;
        }
        data = &data[written as usize..];
    }
}

#[cfg(windows)]
fn write_number_windows(handle: *mut std::ffi::c_void, mut value: u64) {
    let mut buffer = [0u8; 24];
    let mut at = buffer.len();
    loop {
        at -= 1;
        buffer[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    write_all_windows(handle, &buffer[at..]);
}

#[cfg(windows)]
unsafe extern "system" fn windows_exception_handler(_exception: *const std::ffi::c_void) -> i32 {
    let log_handle = LOG_HANDLE.load(Ordering::SeqCst);
    let log_line = LOG_LINE.load(Ordering::SeqCst);
    if !log_handle.is_null() {
        write_all_windows(log_handle, b"panta-native crash: Windows SEH pid=");
        write_number_windows(log_handle, u64::from(std::process::id()));
        write_all_windows(log_handle, b"\n");
    }
    // SAFETY: Windows 在未处理异常回调期间提供标准错误句柄；日志路径指针由
    // 安装期泄露并在进程生命周期内保持有效。
    let stderr = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
    write_all_windows(stderr, b"panta-native crash: Windows SEH pid=");
    write_number_windows(stderr, u64::from(std::process::id()));
    if !log_line.is_null() {
        // SAFETY: 指向安装期泄露的 NUL 结尾缓冲区，进程生命周期内有效。
        let cstr = unsafe { std::ffi::CStr::from_ptr(log_line.cast()) };
        write_all_windows(stderr, cstr.to_bytes());
    }
    // EXCEPTION_CONTINUE_SEARCH：保留 Windows 默认 WER/minidump 处置。
    0
}

/// Windows 不使用 POSIX 信号；注册最小 SEH 顶层过滤器，写出信号等价的
/// 进程级异常记录后继续交给 WER。日志文件仍在安装期预创建并常驻打开。
#[cfg(windows)]
pub fn install_crash_handler(log_dir: &str) -> std::io::Result<PathBuf> {
    let dir = if log_dir.is_empty() {
        std::env::temp_dir()
    } else {
        PathBuf::from(log_dir)
    };
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("panta-native-crash-{}.log", std::process::id()));
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    // SAFETY: 句柄有意常驻进程生命周期；异常过滤器在任意线程触发时写入，
    // 不关闭、不交给其他所有者，进程退出由 Windows 回收。
    let handle = file.into_raw_handle().cast();
    let log_line = std::ffi::CString::new(format!(" log={}\n", path.display()))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "日志路径包含 NUL"))?;
    // SAFETY: CString 以进程生命周期泄露，过滤器只读取其 NUL 结尾字节；重复
    // 安装时旧缓冲区也不能立即释放，因为旧回调可能正在读取它。
    let log_line = log_line.into_raw().cast::<u8>();
    LOG_LINE.store(log_line, Ordering::SeqCst);
    if let Ok(mut current) = LOG_PATH.lock() {
        *current = Some(path.clone());
    }
    LOG_HANDLE.store(handle, Ordering::SeqCst);
    // SAFETY: 注册进程级 Windows 异常过滤器；返回 0 保留默认 WER/minidump。
    unsafe {
        SetUnhandledExceptionFilter(Some(windows_exception_handler));
    }
    Ok(path)
}

/// 当前崩溃日志路径（未安装返回 None）。
#[must_use]
pub fn crash_log_path() -> Option<PathBuf> {
    LOG_PATH.lock().ok().and_then(|current| current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fork 子进程触发 SIGSEGV：处理器应向日志写入信号与 pid，且子进程
    /// 死于原信号（默认处置重发生效）。
    #[cfg(unix)]
    #[test]
    fn handler_logs_segfault_and_reraises() -> std::io::Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "panta-foundation-crash-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir)?;
        let path = install_crash_handler(&dir.to_string_lossy())?;
        assert_eq!(crash_log_path().as_deref(), Some(path.as_path()));

        // SAFETY: fork/raise/waitpid 为本测试的受控进程边界；子进程仅执行
        // raise（处理器内为 async-signal-safe 路径），父进程只 waitpid 与
        // 读取日志文件。
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0, "fork 失败");
        if pid == 0 {
            // SAFETY: raise 与 _exit 均为 async-signal-safe；子进程在此
            // 路径上不返回。
            unsafe {
                libc::raise(libc::SIGSEGV);
                libc::_exit(0);
            }
        }

        let mut status = 0;
        // SAFETY: pid 为本测试 fork 的子进程；status 由 waitpid 初始化。
        unsafe {
            libc::waitpid(pid, &mut status, 0);
        }
        assert!(libc::WIFSIGNALED(status));
        assert_eq!(libc::WTERMSIG(status), libc::SIGSEGV);

        // SIGCHLD 的默认处置是忽略；直接调用处理器可在当前测试进程覆盖
        // 输出与回溯逻辑，而不会破坏下面对 SIGSEGV 重发语义的断言。
        crash_handler(libc::SIGCHLD);

        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("panta-native crash: SIGSEGV"), "{content}");
        assert!(content.contains("pid="), "{content}");
        std::fs::remove_dir_all(&dir).ok();
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn formatting_helpers_write_expected_bytes() -> std::io::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "panta-foundation-crash-format-test-{}",
            std::process::id()
        ));
        let file = std::fs::File::create(&path)?;
        // SAFETY: fd 有效且在读取前保持打开；测试结束时显式关闭，避免泄露到
        // 其他测试进程边界。
        let fd = file.into_raw_fd();
        write_text(fd, "prefix:");
        write_number(fd, 0);
        write_text(fd, ",");
        write_number(fd, 42);
        write_text(fd, ",");
        write_number(fd, u64::MAX);
        // SAFETY: fd 来自本测试刚转移所有权的 File，且仅关闭一次。
        unsafe {
            libc::close(fd);
        }
        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, format!("prefix:0,42,{}", u64::MAX));
        assert_eq!(signal_name(libc::SIGSEGV), "SIGSEGV");
        assert_eq!(signal_name(-1), "UNKNOWN");
        std::fs::remove_file(path).ok();
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn handler_installs_windows_filter() -> std::io::Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "panta-foundation-crash-test-{}",
            std::process::id()
        ));
        let path = install_crash_handler(&dir.to_string_lossy())?;
        assert_eq!(crash_log_path().as_deref(), Some(path.as_path()));
        assert!(path.is_file());
        std::fs::remove_dir_all(&dir).ok();
        Ok(())
    }
}

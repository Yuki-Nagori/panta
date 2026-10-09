//! 托管构建支持公共门面：固定资产、安装事务、编译器与平台环境各归其模块。
//! 下载只在消费者请求工具路径时发生；Cargo 与 CMake 共用已校验的安装树。

mod archive;
mod assets;
mod compiler;
mod environment;
mod install;
mod paths;

pub mod database;
pub mod python;

pub use assets::LLVM_VERSION;
pub use compiler::{
    LlvmCompilers, compiler_rt_dll_dir, resolve_cmake, resolve_llvm_compilers, resolve_ninja,
};
pub use environment::{
    macos_sdk, native_test_env, prepend_path, target_root, use_system_tools, windows_sdk_env,
};
pub use install::{install_directory, install_lock};
pub use paths::exe_name;

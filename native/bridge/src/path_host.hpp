/// 跨平台路径宿主适配（任务 023）：Qt 侧负责标准目录发现与注入、
/// QString↔可往返 UTF-8 转换和 file URL 单次解码；工程引用规则与
/// 包含检查全部在 Rust panta-core，边界不复制规则。
///
/// 契约（modules/paths-and-runtime.md）：
/// - 根类别由宿主显式注入，解析结果恒为绝对路径、与 cwd 无关；
/// - qrc 是只读内置资源，永远不解析为本机路径；
/// - 非 Unicode 路径当前契约不支持：QString→UTF-8 不可往返即拒绝，
///   禁止有损转换后访问其他文件；
/// - CXX Result 在 C++ 侧是异常，适配器捕获并转为 QString 错误，
///   不让异常越过 Qt 事件回调（standards/cxx.md）。
#pragma once

#include "panta_ffi.h"

#include <QString>
#include <QUrl>
#include <memory>

namespace panta::bridge {

class PathHost {
public:
    /// 创建并注入标准目录（user-config→AppConfigLocation、
    /// app-data→AppDataLocation、cache→CacheLocation、
    /// session→TempLocation）。目录为空即失败：不静默回退 cwd。
    /// 测试经 QStandardPaths::setTestModeEnabled 定位到隔离目录。
    [[nodiscard]] static std::unique_ptr<PathHost> create(QString* error);

    /// 注入显式工程根（打开工程时调用）；必须为绝对路径。
    [[nodiscard]] bool setProjectRoot(const QString& root, QString* error);

    /// 纯逻辑解析 `scheme:/relative`；不访问文件系统。
    [[nodiscard]] QString resolve(const QString& reference, QString* error) const;

    /// 读解析：目标必须存在且规范化后在根内（拒绝符号链接/junction 越界）。
    [[nodiscard]] QString resolveExisting(const QString& reference, QString* error) const;

    /// 写目标解析：目标可不存在，最深现存祖先仍须在根内。
    [[nodiscard]] QString resolveWriteTarget(const QString& reference, QString* error) const;

    /// file URL → 本机路径，恰好解码一次（QUrl::toLocalFile）；
    /// 非 file scheme（含 qrc）拒绝为本机路径。
    [[nodiscard]] static QString fileUrlToPath(const QUrl& url, QString* error);

    /// QString → 可跨边界 UTF-8；往返不一致（如未配对代理项）即拒绝。
    /// 这是当前 FFI 契约的非 Unicode 边界，禁止有损转换。
    [[nodiscard]] static bool toBoundaryUtf8(const QString& text, std::string* out, QString* error);

private:
    PathHost() = default;

    /// 统一的调用入口：捕获 rust::Error 并把错误码文本写回 error。
    [[nodiscard]] QString callResolve(
        const QString& reference,
        const std::function<QString(const panta::ffi::PathRef&)>& resolver,
        QString* error) const;

    rust::Box<panta::ffi::PathService> m_service = panta::ffi::path_service_new();
};

} // namespace panta::bridge

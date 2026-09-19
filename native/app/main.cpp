/// Qt 桌面入口（任务 005）：加载 Panta.Shell 模块并运行主窗口。
///
/// 启动序列：先处理 --version（不进事件循环），再把剩余参数交给
/// QGuiApplication（支持 --platform 等标准参数）；QML 加载失败时引擎经
/// qWarning 输出全部错误，本入口以 69（EX_UNAVAILABLE）退出，不静默降级。
/// 窗口生命周期归 QML（ApplicationWindow visible: true），退出走关闭事件。

#include "panta_ffi.h"
#include <QCoreApplication>
#include <QGuiApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QtCore/qnamespace.h>
#include <panta/foundation/version.hpp>
#include <rust/cxx.h>
#ifdef PANTA_ENABLE_BRIDGE_MODULE
#include <QtQml/qqmlextensionplugin.h>
#include <panta/visualization/viewport_backend.hpp>
#endif
#include <cstdio>
#include <string_view>
#ifdef PANTA_ENABLE_BRIDGE_MODULE

Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
Q_IMPORT_QML_PLUGIN(Panta_VisualizationPlugin)
#endif

namespace {

/// 用法错误退出码，与既有约定一致（BSD sysexits.h 的 EX_USAGE）。
constexpr int kExitUsage = 64;
/// UI 资源加载失败退出码（EX_UNAVAILABLE）。
constexpr int kExitUnavailable = 69;

void print_version() {
    const panta::foundation::Version version = panta::foundation::native_version();
    std::printf("panta-native %d.%d.%d\n", version.major, version.minor, version.patch);
}

} // namespace

int main(int argc, char* argv[]) {
    // 原生崩溃此前控制台零输出（任务 047，Rust 实现 panta_foundation::crash）：
    // 先于一切逻辑安装；失败以 qWarning 级别打到 stderr，不静默。
    try {
        panta::ffi::install_crash_handler(rust::String(""));
    } catch (const rust::Error& error) {
        std::fprintf(stderr, "panta-native: 崩溃日志初始化失败：%s\n", error.what());
    }
    for (int index = 1; index < argc; ++index) {
        const std::string_view argument = argv[index];
        if (argument == "--version") {
            print_version();
            return 0;
        }
        if (argument == "--") {
            break;
        }
        // 其余 Qt 标准参数交给 QGuiApplication；明确的未知参数在此拒绝，
        // 避免吞掉拼写错误。
        std::fprintf(
            stderr,
            "panta-native: 未知参数 '%.*s'（用法：panta-native [--version] [-- <Qt 参数>]）\n",
            static_cast<int>(argument.size()), argument.data());
        return kExitUsage;
    }

#ifdef PANTA_ENABLE_BRIDGE_MODULE
    // VTK 视口的图形 API/表面格式选择必须在 QGuiApplication 构造前完成
    // （standards/vtk.md）；中性入口由 VTK 适配器实现。
    panta::visualization::prepare_graphics_environment();
#endif

    const QGuiApplication app(argc, argv);
    // 图形 API 已由 setGraphicsApi 在上方锁定；后端实现（Metal/OpenGL）
    // 由各平台 Qt 运行时决定。

    QQmlApplicationEngine engine;
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app,
        []() { QCoreApplication::exit(kExitUnavailable); }, Qt::QueuedConnection);
#ifdef PANTA_ENABLE_BRIDGE_MODULE
    engine.loadFromModule("Panta.Shell", "App");
#else
    engine.loadFromModule("Panta.Shell", "AppNoBridge");
#endif
    if (engine.rootObjects().isEmpty()) {
        return kExitUnavailable;
    }
    return QGuiApplication::exec();
}

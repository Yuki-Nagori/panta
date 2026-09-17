/// Qt 桌面入口（任务 005）：加载 Panta.Shell 模块并运行主窗口。
///
/// 启动序列：先处理 --version（不进事件循环），再把剩余参数交给
/// QGuiApplication（支持 --platform 等标准参数）；QML 加载失败时引擎经
/// qWarning 输出全部错误，本入口以 69（EX_UNAVAILABLE）退出，不静默降级。
/// 窗口生命周期归 QML（ApplicationWindow visible: true），退出走关闭事件。

#include <panta/foundation/version.hpp>

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QtQml/qqmlextensionplugin.h>
#include <cstdio>
#include <string_view>

Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)

namespace {

/// 用法错误退出码，与既有约定一致（BSD sysexits.h 的 EX_USAGE）。
constexpr int kExitUsage = 64;
/// UI 资源加载失败退出码（EX_UNAVAILABLE）。
constexpr int kExitUnavailable = 69;

void print_version() {
    const panta::foundation::Version version = panta::foundation::native_version();
    std::printf("panta-native %d.%d.%d\n", version.major, version.minor, version.patch);
}

}  // namespace

int main(int argc, char* argv[]) {
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
        std::fprintf(stderr,
                     "panta-native: 未知参数 '%.*s'（用法：panta-native [--version] [-- <Qt 参数>]）\n",
                     static_cast<int>(argument.size()), argument.data());
        return kExitUsage;
    }

    const QGuiApplication app(argc, argv);
    // 图形后端（Metal/OpenGL 等）不在此锁定；视口集成时由任务 007 统一决策。

    QQmlApplicationEngine engine;
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app,
        []() { QCoreApplication::exit(kExitUnavailable); }, Qt::QueuedConnection);
    engine.loadFromModule("Panta.Shell", "App");
    if (engine.rootObjects().isEmpty()) {
        return kExitUnavailable;
    }
    return QGuiApplication::exec();
}

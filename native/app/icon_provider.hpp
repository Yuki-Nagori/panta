#pragma once

class QQmlEngine;

namespace panta {
/// 为 GUI 线程的 Shell 引擎安装内置单色 SVG provider；加载 QML 前调用一次。
/// provider 由引擎拥有，只接受内置图标名与 ARGB 色值，不读取外部路径。
void install_icon_provider(QQmlEngine& engine);
} // namespace panta

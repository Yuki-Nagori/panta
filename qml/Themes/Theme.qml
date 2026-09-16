// 主题单例（qml.md：主题、间距和色彩集中管理）。
pragma Singleton

import QtQuick

QtObject {
    readonly property color colorBackground: "#1e1f22"
    readonly property color colorPanel: "#2b2d31"
    readonly property color colorText: "#e8e8e8"
    readonly property color colorTextMuted: "#9a9da3"
    readonly property color colorAccent: "#4f8cff"
    readonly property color colorError: "#e5534b"

    readonly property int spacingSmall: 8
    readonly property int spacingMedium: 16
    readonly property int spacingLarge: 24

    readonly property int fontTitle: 20
    readonly property int fontBody: 14
}

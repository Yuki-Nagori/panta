// 主题单例（qml.md：主题、间距和色彩集中管理）。
pragma Singleton

import QtQuick

QtObject {
    readonly property color colorBackground: "#f5f7fb"
    readonly property color colorPanel: "#ffffff"
    readonly property color colorText: "#1f2937"
    readonly property color colorTextMuted: "#5f6b7a"
    readonly property color colorAccent: "#2563eb"
    readonly property color colorError: "#c62828"

    readonly property int spacingSmall: 8
    readonly property int spacingMedium: 16
    readonly property int spacingLarge: 24
    readonly property int radiusSmall: 8
    readonly property int controlHeight: 36
    readonly property int windowMinimumWidth: 640
    readonly property int windowMinimumHeight: 480

    readonly property int fontTitle: 20
    readonly property int fontBody: 14
}

// 主题单例（qml.md：主题、间距和色彩集中管理）。
// 默认值即 050 复刻件 ai-docs/qml-html/homepage/homepage.html 的 :root
// 设计 token，是本仓库 UI 的权威设计值；030 起由主题 DSL 提供同名覆盖。
pragma Singleton

import QtQuick

QtObject {
    // 色彩 token（对应复刻件 --color-*）
    readonly property color colorBackground: "#ffffff"
    readonly property color colorPanel: "#ffffff"
    readonly property color colorChrome: "#e9e9e9"
    readonly property color colorChromeLine: "#d0d0d0"
    readonly property color colorPanelLine: "#c9c9c9"
    readonly property color colorStatus: "#efefef"
    readonly property color colorText: "#1c1c1c"
    readonly property color colorTextMuted: "#5a5a5a"
    readonly property color colorIcon: "#4a4a4a"
    readonly property color colorHover: "#e5f1fb"
    readonly property color colorSelected: "#e8f3fd"
    readonly property color colorBrand: "#c8322b"
    readonly property color colorError: "#c62828"
    readonly property color colorMenubar: "#2b2b2b"
    readonly property color colorMenubarText: "#f2f2f2"
    readonly property color colorMenubarHover: "#454545"
    readonly property color colorToolGroupTop: "#b0b0b0"
    readonly property color colorToolGroupBottom: "#666666"
    readonly property color colorRibbonTop: "#e1e1e1"
    readonly property color colorRibbonBottom: "#919191"
    readonly property color colorCloseHover: "#e81123"

    // 字号 token（--font-size-*，逻辑像素）
    readonly property int fontTitle: 13
    readonly property int fontBody: 13
    readonly property int fontSmall: 12

    // 间距 token（--space-* 及复刻件既有间隙值，逻辑像素）
    readonly property int spacingTiny: 2
    readonly property int spacingXSmall: 4
    readonly property int spacingSmall: 6
    readonly property int spacingMedium: 8
    readonly property int spacingLarge: 12
    readonly property int spacingStrip: 10

    // 结构尺寸 token（--size-*：条带高度、控件与图标、栏宽比例，逻辑像素）
    readonly property int titlebarHeight: 30
    readonly property int menubarHeight: 24
    readonly property int ribbonHeight: 84
    readonly property int statusbarHeight: 26
    readonly property int controlHeight: 24
    readonly property int searchHeight: 22
    readonly property int searchFieldWidth: 200
    readonly property int toolbarButtonWidth: 24
    readonly property int toolbarButtonHeight: 22
    readonly property int iconSizeSmall: 13
    readonly property int iconSizeDefault: 15
    readonly property int iconSizeRibbon: 30
    readonly property int caretWidth: 7
    readonly property int caretHeight: 5
    readonly property int logoWidth: 46
    readonly property int logoHeight: 54
    readonly property real leftPanelRatio: 0.26
    readonly property int leftPanelMinimumWidth: 320
    readonly property real outputPanelRatio: 0.42

    // 圆角与线宽 token（--radius-ribbon-tile 及小控件圆角）
    readonly property int radiusSmall: 3
    readonly property int radiusLarge: 4
    readonly property int borderWidth: 1
    // 禁用态整体弱化透明度（qml.md：禁用原因可理解）
    readonly property real disabledOpacity: 0.4

    readonly property int windowMinimumWidth: 640
    readonly property int windowMinimumHeight: 480
}

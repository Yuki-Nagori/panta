// 当前 Shell 的只读主题参数；主题 DSL 接入由任务 030 承接。
pragma Singleton

import QtQuick

QtObject {
    // 界面色彩
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

    // 字号（逻辑像素）
    readonly property int fontTitle: 13
    readonly property int fontBody: 13
    readonly property int fontSmall: 12
    readonly property int fontRibbon: 10

    // 间距（逻辑像素）
    readonly property int spacingTiny: 2
    readonly property int spacingXSmall: 4
    readonly property int spacingSmall: 6
    readonly property int spacingMedium: 8
    readonly property int spacingLarge: 12
    readonly property int spacingStrip: 10

    // 结构尺寸（逻辑像素）与栏宽比例
    readonly property int titlebarHeight: 30
    readonly property int menubarHeight: 24
    readonly property int ribbonHeight: 96
    readonly property int ribbonToolMinimumWidth: 56
    readonly property int ribbonToolHeight: 68
    readonly property int ribbonLabelHeight: 22
    readonly property int ribbonContentSpacing: 3
    readonly property int ribbonTextLineHeight: 12
    readonly property int ribbonTextLines: 2
    readonly property int ribbonCaretHeight: 5
    readonly property int ribbonTrailingWidth: 64
    readonly property int statusbarHeight: 26
    readonly property int controlHeight: 24
    readonly property int searchHeight: 22
    readonly property int searchFieldWidth: 200
    readonly property int toolbarButtonWidth: 24
    readonly property int toolbarButtonHeight: 22
    readonly property int iconSizeSmall: 16
    readonly property int iconSizeDefault: 18
    readonly property int iconSizeRibbon: 26
    readonly property int iconSizeCaret: 12
    readonly property int paneCloseSize: 20
    readonly property int tabSlideDuration: 120
    readonly property int tabSegmentWidth: 96
    readonly property int panelToolbarHeight: 32
    readonly property int titlebarMinimumContentWidth: 1100
    readonly property int toolTipDelay: 600
    readonly property int logoWidth: 46
    readonly property real leftPanelRatio: 0.26
    readonly property int leftPanelMinimumWidth: 320
    readonly property real outputPanelRatio: 0.42

    // 圆角与线宽
    readonly property int radiusSmall: 3
    readonly property int borderWidth: 1
    // 禁用控件的整体透明度
    readonly property real disabledOpacity: 0.4

    readonly property int windowMinimumWidth: 640
    readonly property int windowMinimumHeight: 480
}

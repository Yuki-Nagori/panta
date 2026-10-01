// 当前 Shell 的只读主题参数；主题 DSL 接入由任务 030 承接。
pragma Singleton

import QtQuick
import Panta.Shell

QtObject {
    // 界面色彩
    readonly property color colorBackground: "#ffffff"
    readonly property color colorPanel: "#ffffff"
    readonly property color colorTransparent: "transparent"
    readonly property color colorChrome: "#e9e9e9"
    readonly property color colorChromeLine: "#d0d0d0"
    readonly property color colorPanelLine: "#c9c9c9"
    readonly property color colorStatus: "#efefef"
    readonly property color colorText: "#1c1c1c"
    readonly property color colorTextMuted: "#5a5a5a"
    readonly property color colorIcon: "#4a4a4a"
    readonly property color colorHover: "#e5f1fb"
    readonly property color colorSelected: "#e8f3fd"
    readonly property color colorFocus: "#2e7ce0"
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
    readonly property color colorDocumentBand: "#e7e4e1"
    readonly property color colorDocumentBandLine: "#c8c4c0"
    readonly property color colorDocumentHover: "#f0eeec"
    readonly property color colorDocumentClosePressed: "#d5e3ef"
    readonly property color colorDocumentWelcomeIcon: "#a83e47"
    readonly property color colorDialogPrimary: "#e5f1fb"
    readonly property color colorDialogPrimaryBorder: "#5d9fc7"

    // 字体与字号
    readonly property string analysisLogFontFamily: PlatformFonts.fixedFamily
    readonly property int fontTitle: 13
    readonly property int fontBody: 13
    readonly property int fontSmall: 12
    readonly property int fontRibbon: 10

    // 通用间距（逻辑像素）
    readonly property int spacingTiny: 2
    readonly property int spacingXSmall: 4
    readonly property int spacingSmall: 6
    readonly property int spacingMedium: 8
    readonly property int spacingLarge: 12
    readonly property int spacingStrip: 10

    // 通用图标尺寸（逻辑像素）
    readonly property int iconSizeSmall: 16
    readonly property int iconSizeDefault: 18
    readonly property int iconSizeRibbon: 26
    readonly property int iconSizeCompact: 12
    readonly property int iconSizePaneClose: 10

    // 圆角与边框（逻辑像素）
    readonly property int radiusSmall: 3
    readonly property int borderWidth: 1
    readonly property int focusBorderWidth: 2

    // 通用控件尺寸（逻辑像素）
    readonly property int comboBoxHeight: 30
    readonly property int controlHeight: 24
    readonly property int selectionIndicatorSize: 16
    readonly property int searchHeight: 22
    readonly property int searchFieldWidth: 200
    readonly property int toolbarButtonWidth: 24
    readonly property int toolbarButtonHeight: 22
    readonly property int paneCloseSize: 20
    readonly property int tabSegmentWidth: 96

    // Shell 条带与标题布局（逻辑像素）
    readonly property int chromeImplicitWidth: 800
    readonly property int titlebarHeight: 30
    readonly property int menubarHeight: 24
    readonly property int statusbarHeight: 26
    readonly property int titlebarMinimumContentWidth: 1100
    readonly property int logoWidth: 46

    // Ribbon 布局（尺寸为逻辑像素，行数为计数）
    readonly property int ribbonHeight: 96
    readonly property int ribbonToolMinimumWidth: 56
    readonly property int ribbonToolHeight: 68
    readonly property int ribbonLabelHeight: 22
    readonly property int ribbonContentSpacing: 3
    readonly property int ribbonTextLineHeight: 12
    readonly property int ribbonTextLines: 2
    readonly property int ribbonCaretHeight: 5
    readonly property int ribbonTrailingWidth: 64
    readonly property int ribbonCompactHeight: 22
    readonly property int ribbonCompactWidth: 24
    readonly property int ribbonCompactIconSize: 20
    readonly property int ribbonResultsControlWidth: 210
    readonly property int ribbonTimeDisplayWidth: 44
    readonly property int ribbonScaleValueWidth: 28

    // 侧栏布局（尺寸为逻辑像素，比例无量纲）
    readonly property int panelToolbarHeight: 32
    readonly property int leftPanelMinimumWidth: 320
    readonly property int layersTabWidth: 92
    readonly property real leftPanelRatio: 0.26
    readonly property real layersPanelRatio: 0.42
    readonly property real projectTreeHeightRatio: 0.36

    // 文档页签（逻辑像素）
    readonly property int documentTabWelcomeIconSize: 17
    readonly property int documentTabRadius: 8
    readonly property int documentTabCloseRadius: 5
    readonly property int documentTabBottomLineHeight: 3
    readonly property int documentTabWidth: 130
    readonly property int documentTabBarHeight: 39
    readonly property int documentTabContentInset: 11
    readonly property int documentTabCloseSize: 19
    readonly property int documentTabCloseRightInset: 3

    // 弹窗与表单控件（逻辑像素）
    readonly property int dialogActionWidth: 92
    readonly property int dialogActionSpacing: 5
    readonly property int formControlHeight: 30
    readonly property int profilePlotHeight: 140
    readonly property int profileStepColumnWidth: 40
    readonly property int processValueColumnWidth: 126

    // 日志面板（尺寸为逻辑像素，比例无量纲）
    readonly property int analysisLogHeight: 220
    readonly property int analysisLogRunWidth: 100
    readonly property real analysisLogMaximumRatio: 0.45

    // 交互阈值（逻辑像素）与时长（毫秒）
    readonly property int tabSlideDuration: 120
    readonly property int documentTabDragThreshold: 4
    readonly property int documentTabAnimationDuration: 150
    readonly property int toolTipDelay: 600

    // 禁用状态
    readonly property real disabledOpacity: 0.4

    // 主窗口最小尺寸（逻辑像素）
    readonly property int windowMinimumWidth: 640
    readonly property int windowMinimumHeight: 480

    // 新建工程窗口尺寸与最小约束（逻辑像素）
    readonly property int newProjectDialogWidth: 620
    readonly property int newProjectDialogHeight: 360
    readonly property int newProjectDialogMinimumWidth: 520
    readonly property int newProjectDialogMinimumHeight: 320

    // 导入窗口尺寸与最小约束（逻辑像素）
    readonly property int importDialogWidth: 680
    readonly property int importDialogHeight: 430
    readonly property int importDialogMinimumWidth: 580
    readonly property int importDialogMinimumHeight: 360

    // 分析序列窗口尺寸与最小约束（逻辑像素）
    readonly property int analysisSequenceDialogWidth: 560
    readonly property int analysisSequenceDialogHeight: 380
    readonly property int analysisSequenceDialogMinimumWidth: 420
    readonly property int analysisSequenceDialogMinimumHeight: 340

    // 材料选择窗口尺寸与最小约束（逻辑像素）
    readonly property int materialDialogWidth: 680
    readonly property int materialDialogHeight: 540
    readonly property int materialDialogMinimumWidth: 640
    readonly property int materialDialogMinimumHeight: 460

    // Fill 工艺设置窗口尺寸与最小约束（逻辑像素）
    readonly property int fillSettingsDialogWidth: 720
    readonly property int fillSettingsDialogHeight: 600
    readonly property int fillSettingsDialogMinimumWidth: 660
    readonly property int fillSettingsDialogMinimumHeight: 480

    // Gate Location 工艺设置窗口尺寸与最小约束（逻辑像素）
    readonly property int gateLocationSettingsDialogWidth: 720
    readonly property int gateLocationSettingsDialogHeight: 480
    readonly property int gateLocationSettingsDialogMinimumWidth: 660
    readonly property int gateLocationSettingsDialogMinimumHeight: 400

    // 保压曲线窗口尺寸与最小约束（逻辑像素）
    readonly property int holdingProfileDialogWidth: 540
    readonly property int holdingProfileDialogHeight: 400
    readonly property int holdingProfileDialogMinimumWidth: 480
    readonly property int holdingProfileDialogMinimumHeight: 320
}

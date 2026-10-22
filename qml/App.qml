// Shell 页面装配；页面持有 ViewModel 和对话框，面板只发语义命令信号。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Panta.Bridge
import Panta.Shell as Shell

ApplicationWindow {
    id: shellWindow
    minimumWidth: Theme.windowMinimumWidth
    minimumHeight: Theme.windowMinimumHeight
    width: Theme.windowMinimumWidth
    height: Theme.windowMinimumHeight
    // Linux XCB 会把空标题替换为应用名；零宽字符让窗口管理器保留原生标题栏按钮，
    // 同时不显示文字。应用显示名在 Linux 入口中显式留空，避免 QPA 追加名称。
    title: Qt.platform.os === "linux" ? "\u200B" : ""
    // Windows 隐藏系统图标/标题但保留 Qt 原生控制按钮。Linux 显式保留空标题
    // 和系统按钮提示，使 XCB 保留窗口管理器装饰及操作按钮。
    flags: Qt.Window | Qt.ExpandedClientAreaHint | Qt.NoTitleBarBackgroundHint | (Qt.platform.os === "windows" ? Qt.CustomizeWindowHint | Qt.WindowMinimizeButtonHint | Qt.WindowMaximizeButtonHint | Qt.WindowCloseButtonHint : Qt.platform.os === "linux" ? Qt.CustomizeWindowHint | Qt.WindowTitleHint | Qt.WindowMinimizeButtonHint | Qt.WindowMaximizeButtonHint | Qt.WindowCloseButtonHint : 0)
    topPadding: 0
    leftPadding: 0
    rightPadding: 0
    visible: true
    property int lastWindowedWidth: Settings.windowWidth
    property int lastWindowedHeight: Settings.windowHeight
    property bool lastRestorableMaximized: Settings.windowMaximized
    property alias settingsRouter: settingsPageRouter
    color: Theme.colorPanel
    onClosing: event => {
        if (projectModel.materialConfirmationPending || projectModel.fillSettingsConfirmationPending || projectModel.gateLocationSettingsConfirmationPending)
            event.accepted = false;
        else
            Settings.saveWindowState(Qt.size(lastWindowedWidth, lastWindowedHeight), lastRestorableMaximized);
    }
    onVisibilityChanged: function (visibility) {
        if (visibility === Window.Maximized)
            lastRestorableMaximized = true;
        else if (visibility === Window.Windowed)
            lastRestorableMaximized = false;
    }
    onWidthChanged: function (width) {
        if (shellWindow.visibility === Window.Windowed)
            lastWindowedWidth = width;
    }
    onHeightChanged: function (height) {
        if (shellWindow.visibility === Window.Windowed)
            lastWindowedHeight = height;
    }

    menuBar: Qt.platform.os === "osx" ? platformMenuBar : null

    PlatformMenuBar {
        id: platformMenuBar
        objectName: "platformApplicationMenuBar"
        visible: Qt.platform.os === "osx"
        hostWindow: shellWindow
        projectCommandsEnabled: true
        onSettingsRequested: settingsPageRouter.openPage("general")
        onQuitRequested: shellWindow.close()
        onNewProjectRequested: newProjectDialog.open()
        onOpenProjectRequested: openProjectFileDialog.open()
    }

    SettingsPageRouter {
        id: settingsPageRouter
        objectName: "settingsPageRouter"
        ownerWindow: shellWindow
        settingsService: Settings
    }
    readonly property bool projectOpen: projectModel.currentPath.length > 0
    readonly property real nativeControlsLeftInset: WindowChromeMetrics.leadingSystemControlsInset
    readonly property bool layersDockShown: layersPanel.dockOpen
    readonly property string statusMessage: projectModel.error.length > 0 ? projectModel.error : Settings.settingsError.length > 0 ? qsTranslate("App", "Some application preferences could not be applied. (%1)").arg(Settings.settingsError.split(":")[0]) : ""
    // 展示状态独立于工程快照，浏览开始页不卸载工程或视口。
    property string activeRibbonTab: "start-learn"

    Component.onCompleted: {
        // 只在启动时读取一次；后续偏好快照变更不能通过 QML binding 改写用户正在调整的窗口。
        shellWindow.width = Settings.windowWidth;
        shellWindow.height = Settings.windowHeight;
        shellWindow.visibility = Settings.windowMaximized ? Window.Maximized : Window.Windowed;
        WindowChromeMetrics.observeWindow(shellWindow);
        Shell.ThemeViewModel.selectTheme(Settings.themeId);
    }

    Connections {
        target: Shell.ThemeViewModel
        function onActiveThemeChanged() {
            if (!Settings.setThemeId(Shell.ThemeViewModel.activeTheme))
                Shell.ThemeViewModel.selectTheme(Settings.themeId);
        }
    }

    function selectRibbonTab(tab) {
        // 工程工具仅在打开工程后可达，开始页始终可达。
        if (tab === "start-learn" || ((tab === "home" || tab === "results" || tab === "tools" || tab === "view") && projectOpen))
            activeRibbonTab = tab;
    }

    ProjectViewModel {
        id: projectModel
        objectName: "projectModel"
        // 仅成功创建 / 打开时导航；改名、保存和失败不打断当前页签。
        onProjectCreated: path => {
            Settings.recordRecentProject(path);
            shellWindow.selectRibbonTab("home");
        }
        onProjectOpened: path => {
            Settings.recordRecentProject(path);
            shellWindow.selectRibbonTab("home");
        }
        onProjectImported: layersPanel.dockOpen = true
        onGateLocationSettingsConfirmationFinished: succeeded => {
            if (succeeded)
                gateLocationSettingsDialog.close();
        }
        onFillSettingsConfirmationFinished: succeeded => {
            if (succeeded)
                fillSettingsDialog.close();
        }
        onMaterialConfirmationFinished: succeeded => {
            if (succeeded)
                materialDialog.close();
        }
        onProjectOperationFinished: (operation, succeeded) => {
            if (operation === "analysis-sequence" && succeeded)
                analysisSequenceDialog.close();
        }
    }

    Connections {
        target: Settings
        function onLanguageChanged() {
            projectModel.retranslateError();
        }
    }

    NewProjectDialog {
        id: newProjectDialog
        ownerWindow: shellWindow
        projectModel: projectModel
    }

    ImportDialog {
        id: importDialog
        ownerWindow: shellWindow
        projectModel: projectModel
    }

    AnalysisSequenceDialog {
        id: analysisSequenceDialog
        ownerWindow: shellWindow
        sequences: projectModel.analysisSequences
        planSettings: projectModel.planSettings
        errorText: projectModel.error
        saving: projectModel.projectOperationPending
        onSelectionRequested: (path, revision, importId, sequenceId) => {
            projectModel.setAnalysisSequence(path, revision, importId, sequenceId);
        }
    }

    GateLocationSettingsDialog {
        id: gateLocationSettingsDialog
        ownerWindow: shellWindow
        planSettings: projectModel.planSettings
        validationModel: projectModel
        saving: projectModel.gateLocationSettingsConfirmationPending
        errorText: projectModel.error
        onSettingsRequested: (path, revision, importId, settings) => projectModel.setGateLocationSettings(path, revision, importId, settings)
    }

    FillProcessSettingsDialog {
        id: fillSettingsDialog
        ownerWindow: shellWindow
        planSettings: projectModel.planSettings
        validationModel: projectModel
        saving: projectModel.fillSettingsConfirmationPending
        errorText: projectModel.error
        onSettingsRequested: (path, revision, importId, settings) => {
            projectModel.setFillSettings(path, revision, importId, settings);
        }
    }

    MaterialSelectionDialog {
        id: materialDialog
        ownerWindow: shellWindow
        material: projectModel.defaultMaterial
        planSettings: projectModel.planSettings
        errorText: projectModel.error
        saving: projectModel.materialConfirmationPending
        onSelectionRequested: (path, revision, importId, materialId) => {
            projectModel.setMaterial(path, revision, importId, materialId);
        }
    }

    FileDialog {
        id: openProjectFileDialog
        objectName: "openProjectFileDialog"
        title: qsTranslate("IconActionOpenProject", "Open Project")
        currentFolder: projectModel.defaultLocationUrl
        fileMode: FileDialog.OpenFile
        nameFilters: [qsTranslate("ProjectFileDialog", "Panta project files (*.panta)")]
        onAccepted: {
            projectModel.openProjectUrl(selectedFile);
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        TopChromePanel {
            objectName: "titleChromePanel"
            Layout.fillWidth: true
            caption: shellWindow.projectOpen ? "panta 2027 · " + projectModel.currentName : "panta 2027"
            projectOpen: shellWindow.projectOpen
            activeRibbonTab: shellWindow.activeRibbonTab
            nativeControlsLeftInset: shellWindow.nativeControlsLeftInset
            nativeControlsRightInset: WindowChromeMetrics.trailingSystemControlsInset
            hostWindow: shellWindow
            recentProjects: Settings.recentProjectPaths
            onMenuRequested: key => shellWindow.selectRibbonTab(key)
            onNewProjectRequested: newProjectDialog.open()
            onOpenProjectRequested: openProjectFileDialog.open()
            onRecentProjectRequested: path => projectModel.openProject(path)
            onSettingsRequested: settingsPageRouter.openPage("general")
            onExitRequested: shellWindow.close()
        }

        RibbonPanel {
            Layout.fillWidth: true
            activeRibbonTab: shellWindow.activeRibbonTab
            onOpenProjectRequested: openProjectFileDialog.open()
            onNewProjectRequested: newProjectDialog.open()
            onImportRequested: importDialog.open()
            onResultsRequested: shellWindow.selectRibbonTab("results")
            onLogsRequested: viewportPane.logsOpen = !viewportPane.logsOpen
        }

        // 先确定左栏比例宽度，再把剩余区域交给原生视口。
        Item {
            id: workspace

            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                id: leftColumn
                readonly property real panelContentHeight: Math.max(0, height - (shellWindow.layersDockShown ? Theme.borderWidth : 0))

                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                // 栏宽取工作区宽的比例，下限 leftPanelMinimumWidth；锚定工作区
                // 宽而非自身宽，避免首选宽与分配宽互相依赖成绑定环。
                width: Math.max(workspace.width * Theme.leftPanelRatio, Theme.leftPanelMinimumWidth)
                spacing: 0

                SidebarPanel {
                    id: sidebarPanel
                    objectName: "sidebarPanel"

                    Layout.fillWidth: true
                    Layout.fillHeight: !shellWindow.layersDockShown
                    Layout.preferredHeight: shellWindow.layersDockShown ? leftColumn.panelContentHeight * (1 - Theme.layersPanelRatio) : 0
                    projectOpen: shellWindow.projectOpen
                    projectName: projectModel.currentName
                    logsOpen: viewportPane.logsOpen
                    logRunCount: viewportPane.logRunCount
                    resultRun: viewportPane.selectedLogRun
                    onLogsRequested: viewportPane.logsOpen = !viewportPane.logsOpen
                    importedPartNames: projectModel.importedPartNames
                    importedPartIds: projectModel.importedPartIds
                    activeDocumentId: projectModel.activeDocumentId
                    activeDocumentTitle: projectModel.activeDocumentTitle
                    analysisSequenceText: qsTranslate("AnalysisSequence", projectModel.planSettings.sequenceSourceText ?? "")
                    analysisSequenceId: projectModel.planSettings.sequenceId ?? ""
                    materialText: qsTranslate("Material", projectModel.planSettings.materialSourceText ?? "")
                    materialId: projectModel.planSettings.materialId ?? ""
                    fillSettingsConfirmed: projectModel.planSettings.fillSettingsConfirmed ?? false
                    gateLocationSettingsConfirmed: projectModel.planSettings.gateLocationSettingsConfirmed ?? false
                    onProcessSettingsRequested: {
                        projectModel.clearError();
                        if (projectModel.planSettings.sequenceId === "gate-location")
                            gateLocationSettingsDialog.open();
                        else
                            fillSettingsDialog.open();
                    }
                    onMaterialRequested: {
                        projectModel.clearError();
                        materialDialog.open();
                    }
                    meshType: projectModel.planSettings.meshType ?? ""
                    meshTypes: projectModel.meshTypes
                    onAnalysisSequenceRequested: {
                        projectModel.clearError();
                        analysisSequenceDialog.open();
                    }
                    onOpenImportRequested: recordId => projectModel.openImportRecord(recordId)
                    onCloseRequested: sidebarPanel.visible = false
                    onOpenProjectRequested: openProjectFileDialog.open()
                    onNewProjectRequested: newProjectDialog.open()
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Theme.borderWidth
                    visible: shellWindow.layersDockShown
                    color: Theme.colorPanelLine
                }

                LayersPanel {
                    id: layersPanel
                    objectName: "layersPanel"
                    Layout.fillWidth: true
                    Layout.fillHeight: shellWindow.layersDockShown
                    Layout.preferredHeight: shellWindow.layersDockShown ? leftColumn.panelContentHeight * Theme.layersPanelRatio : 0
                    importedPartNames: projectModel.importedPartNames
                    onCloseRequested: dockOpen = false
                }
            }

            Rectangle {
                id: workspaceSplit

                anchors.left: leftColumn.right
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: Theme.borderWidth
                color: Theme.colorPanelLine
            }

            ViewportPane {
                id: viewportPane

                meshSource: projectModel
                reducedMotion: Settings.reducedMotion
                logContextId: projectModel.currentPath + ":" + (projectModel.planSettings.importId ?? "")
                anchors.left: workspaceSplit.right
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                onCloseRequested: viewportPane.visible = false
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.statusbarHeight
            color: Theme.colorStatus

            Rectangle {
                anchors.top: parent.top
                width: parent.width
                height: Theme.borderWidth
                color: Theme.colorChromeLine
            }

            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                anchors.leftMargin: Theme.spacingStrip

                ThemedLabel {
                    Layout.fillWidth: true
                    text: shellWindow.statusMessage.length > 0 ? shellWindow.statusMessage : qsTr("Ready")
                    textSize: Theme.fontSmall
                    textColor: shellWindow.statusMessage.length > 0 ? Theme.colorError : Theme.colorTextMuted
                    elide: Text.ElideRight
                }
            }
        }
    }
}

// Shell 页面装配；页面持有 ViewModel 和对话框，面板只发语义命令信号。
// 窗口控制由系统标题栏承接。
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Panta.Bridge

ApplicationWindow {
    id: shellWindow
    minimumWidth: Theme.windowMinimumWidth
    minimumHeight: Theme.windowMinimumHeight
    visible: true
    visibility: Window.Maximized
    title: qsTr("panta")
    color: Theme.colorPanel
    readonly property bool projectOpen: projectModel.currentPath.length > 0
    readonly property bool layersDockShown: layersPanel.dockOpen
    readonly property string statusMessage: projectModel.error.length > 0 ? projectModel.error : viewModel.error
    // 展示状态独立于工程快照，浏览开始页不卸载工程或视口。
    property string activeRibbonTab: "start-learn"

    function selectRibbonTab(tab) {
        // 页面统一校验可达状态；未接入的菜单不能产生空白或无工程的 Home。
        if (tab === "start-learn" || (tab === "home" && projectOpen))
            activeRibbonTab = tab;
    }

    ShellViewModel {
        id: viewModel
    }

    ProjectViewModel {
        id: projectModel
        objectName: "projectModel"
        // 仅成功创建 / 打开时导航；改名、保存和失败不打断当前页签。
        onProjectCreated: shellWindow.selectRibbonTab("home")
        onProjectOpened: shellWindow.selectRibbonTab("home")
        onProjectImported: layersPanel.dockOpen = true
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

    FileDialog {
        id: openProjectFileDialog
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
            Layout.fillWidth: true
            caption: shellWindow.projectOpen ? "panta 2027 · " + projectModel.currentName : viewModel.caption.length > 0 ? viewModel.caption : "panta 2027"
            projectOpen: shellWindow.projectOpen
            activeRibbonTab: shellWindow.activeRibbonTab
            onMenuRequested: key => shellWindow.selectRibbonTab(key)
        }

        RibbonPanel {
            Layout.fillWidth: true
            activeRibbonTab: shellWindow.activeRibbonTab
            onOpenProjectRequested: openProjectFileDialog.open()
            onNewProjectRequested: newProjectDialog.open()
            onImportRequested: importDialog.open()
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

                TasksPanel {
                    id: tasksPanel
                    objectName: "tasksPanel"

                    Layout.fillWidth: true
                    Layout.fillHeight: !shellWindow.layersDockShown
                    Layout.preferredHeight: shellWindow.layersDockShown ? leftColumn.panelContentHeight * (1 - Theme.layersPanelRatio) : 0
                    projectOpen: shellWindow.projectOpen
                    projectName: projectModel.currentName
                    importedPartNames: projectModel.importedPartNames
                    importedPartName: projectModel.importedPartName
                    onCloseRequested: tasksPanel.visible = false
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

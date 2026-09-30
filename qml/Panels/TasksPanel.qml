// 工程 / 任务 Dock；上方列出工程和 STL，下方展示最新导入零件的任务。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

PanelSurface {
    id: panel

    property bool projectOpen: false
    property string projectName: ""
    property var importedPartNames: []
    // 稳定导入记录 ID，与 importedPartNames 按下标对应；行点击即请求
    // 打开 / 激活对应视口文档（080）。
    property var importedPartIds: []
    property string activeDocumentId: ""
    property string activeDocumentTitle: ""
    property bool importedPartAvailable: importedPartNames.length > 0
    property string importedPartName: ""
    readonly property var planTaskItems: [
        {
            id: "imported-part",
            text: qsTranslate("ImportTask", "Part (%1)").arg(panel.activePartTitle),
            icon: "stl-file",
            completed: true
        },
        {
            id: "create-mesh",
            text: qsTranslate("ImportTask", "Create Mesh..."),
            icon: "task-mesh",
            completed: false
        },
        {
            id: "fill",
            text: qsTranslate("ImportTask", "Fill"),
            icon: "task-fill",
            completed: false
        },
        {
            id: "material-data",
            text: qsTranslate("ImportTask", "Material Data"),
            icon: "task-material",
            completed: false
        },
        {
            id: "injection-locations",
            text: qsTranslate("ImportTask", "Set Injection Locations..."),
            icon: "task-injection",
            completed: false
        },
        {
            id: "process-settings",
            text: qsTranslate("ProcessTask", "Process Settings (Default)"),
            icon: "task-settings",
            completed: false
        },
        {
            id: "optimization",
            text: qsTranslate("ImportTask", "Optimization (None)"),
            icon: "task-optimization",
            completed: false
        },
        {
            id: "analyze",
            text: qsTranslate("UiCommonAnalysis", "Analyze"),
            icon: "task-analysis",
            completed: false,
            enabled: false
        },
        {
            id: "logs",
            text: qsTranslate("ImportTask", "Logs*"),
            icon: "log",
            completed: false
        }
    ]

    signal closeRequested
    signal openProjectRequested
    signal newProjectRequested
    signal openImportRequested(string recordId)

    // Welcome 或空白视口没有对应导入记录，此时仍显示最近导入项。
    readonly property string activePartTitle: importedPartIds.indexOf(activeDocumentId) >= 0 ? activeDocumentTitle : importedPartName

    implicitWidth: Theme.leftPanelMinimumWidth

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        PanelTabBar {
            id: panelTabs
            Layout.fillWidth: true
            rightPadding: Theme.paneCloseSize + 2 * Theme.spacingXSmall
            tabs: [qsTranslate("TaskPanelTitle", "Tasks"), qsTranslate("UiCommonNavigation", "Tools"), qsTranslate("UiCommonNavigation", "Shared Views")]
        }

        StackLayout {
            id: dockContent
            objectName: "tasksPanelPages"
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: panelTabs.currentIndex

            ColumnLayout {
                spacing: 0

                ListView {
                    id: projectTree
                    objectName: "projectTreeSection"
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.max(0, dockContent.height - (panel.projectOpen ? Theme.borderWidth : 0)) * 0.36
                    clip: true
                    model: panel.importedPartNames
                    reuseItems: true
                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                    }

                    header: ColumnLayout {
                        width: projectTree.width
                        spacing: 0

                        // Tasks 的空工程入口；工程打开后由工程树和 Plan tasks 接替。
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 0
                            visible: !panel.projectOpen

                            ThemedToolButton {
                                objectName: "openProjectTaskAction"
                                Layout.fillWidth: true
                                text: qsTranslate("IconActionOpenProject", "Open Project")
                                dimText: "…"
                                iconName: "open"
                                preserveIconColors: true
                                iconSize: Theme.iconSizeSmall
                                contentAlignLeft: true
                                contentColor: Theme.colorText
                                contentPadding: Theme.spacingLarge
                                hoverColor: Theme.colorHover
                                onClicked: panel.openProjectRequested()
                            }
                            ThemedToolButton {
                                objectName: "newProjectTaskAction"
                                Layout.fillWidth: true
                                text: qsTranslate("UiCommonNavigation", "New Project")
                                dimText: "…"
                                iconName: "new"
                                preserveIconColors: true
                                iconSize: Theme.iconSizeSmall
                                contentAlignLeft: true
                                contentColor: Theme.colorText
                                contentPadding: Theme.spacingLarge
                                hoverColor: Theme.colorHover
                                onClicked: panel.newProjectRequested()
                            }
                        }

                        ThemedToolButton {
                            id: projectEntry
                            objectName: "projectTaskItem"
                            Layout.fillWidth: true
                            visible: panel.projectOpen
                            text: qsTranslate("ProjectTaskItem", "Project '%1'").arg(panel.projectName)
                            iconName: "project-file"
                            preserveIconColors: true
                            iconSize: Theme.iconSizeSmall
                            contentAlignLeft: true
                            contentColor: Theme.colorText
                            contentPadding: Theme.spacingLarge
                            hoverColor: Theme.colorHover
                            contentItem: RowLayout {
                                spacing: Theme.spacingMedium
                                ThemedIcon {
                                    name: projectEntry.iconName
                                    iconSize: projectEntry.iconSize
                                    preserveSourceColors: true
                                    color: projectEntry.contentColor
                                }
                                ThemedLabel {
                                    Layout.fillWidth: true
                                    text: projectEntry.text
                                    textSize: projectEntry.font.pixelSize
                                    elide: Text.ElideRight
                                }
                            }
                        }
                    }

                    delegate: Rectangle {
                        id: importedPartEntry
                        objectName: "importedPartEntry"
                        required property var modelData
                        required property int index

                        readonly property string recordId: panel.importedPartIds[index] ?? ""
                        readonly property bool isActiveDocument: recordId !== "" && recordId === panel.activeDocumentId

                        width: projectTree.width
                        height: Theme.controlHeight
                        color: isActiveDocument ? Theme.colorSelected : partHover.hovered ? Theme.colorDocumentHover : Theme.colorTransparent

                        HoverHandler {
                            id: partHover
                            acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
                        }
                        TapHandler {
                            id: partTap
                            gesturePolicy: TapHandler.ReleaseWithinBounds
                            onTapped: panel.openImportRequested(importedPartEntry.recordId)
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Theme.spacingLarge * 2
                            anchors.rightMargin: Theme.spacingSmall
                            spacing: Theme.spacingSmall

                            ThemedIcon {
                                name: "stl-file"
                                iconSize: Theme.iconSizeSmall
                                preserveSourceColors: true
                            }
                            ThemedLabel {
                                Layout.fillWidth: true
                                text: importedPartEntry.modelData
                                textSize: Theme.fontBody
                                elide: Text.ElideRight
                            }
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Theme.borderWidth
                    visible: panel.projectOpen
                    color: Theme.colorPanelLine
                }

                ScrollView {
                    id: planTaskSection
                    objectName: "planTasksSection"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.preferredHeight: Math.max(0, dockContent.height - (panel.projectOpen ? Theme.borderWidth : 0)) * 0.64
                    clip: true
                    contentWidth: availableWidth
                    ScrollBar.vertical.policy: ScrollBar.AsNeeded

                    ColumnLayout {
                        width: planTaskSection.availableWidth
                        spacing: 0
                        visible: panel.projectOpen && panel.importedPartAvailable

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: Theme.panelToolbarHeight
                            color: Theme.colorChrome

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Theme.spacingMedium
                                anchors.rightMargin: Theme.paneCloseSize + Theme.spacingMedium
                                spacing: Theme.spacingSmall

                                ThemedIcon {
                                    name: "plan-tasks"
                                    iconSize: Theme.iconSizeSmall
                                    preserveSourceColors: true
                                }
                                ThemedLabel {
                                    Layout.fillWidth: true
                                    text: qsTranslate("ProjectTaskItem", "Plan tasks: %1").arg(panel.activePartTitle)
                                    textSize: Theme.fontBody
                                    elide: Text.ElideRight
                                }
                            }
                        }

                        Repeater {
                            model: panel.planTaskItems

                            delegate: RowLayout {
                                id: taskRow
                                required property var modelData
                                objectName: "planTaskItem_" + taskRow.modelData.id

                                Layout.fillWidth: true
                                Layout.leftMargin: Theme.spacingMedium
                                Layout.rightMargin: Theme.spacingSmall
                                spacing: Theme.spacingSmall

                                Item {
                                    Layout.preferredWidth: Theme.iconSizeSmall
                                    Layout.preferredHeight: Theme.iconSizeSmall

                                    ThemedIcon {
                                        anchors.centerIn: parent
                                        visible: taskRow.modelData.completed === true
                                        name: "status-ok"
                                        iconSize: Theme.iconSizeSmall
                                        preserveSourceColors: true
                                    }
                                }

                                ThemedToolButton {
                                    Layout.fillWidth: true
                                    text: taskRow.modelData.text
                                    iconName: taskRow.modelData.icon
                                    preserveIconColors: true
                                    iconSize: Theme.iconSizeSmall
                                    enabled: taskRow.modelData.enabled !== false
                                    contentAlignLeft: true
                                    contentColor: Theme.colorText
                                    contentPadding: Theme.spacingSmall
                                    hoverColor: Theme.colorHover
                                }
                            }
                        }
                    }
                }
            }

            Item {
                objectName: "tasksToolsPage"

                InformationPanel {
                    objectName: "tasksToolsInformation"
                    anchors.fill: parent
                    titleText: qsTranslate("TaskPanelInformation", "Information")
                    messages: [qsTranslate("TaskPanelToolsHelp", "Use the tools above to access each tool."), qsTranslate("TaskPanelToolsHelp", "Open a tool's help to learn how to use it."), qsTranslate("TaskPanelToolsHelp", "Hold Ctrl while clicking to select multiple entities.")]
                }
            }

            Item {
                objectName: "sharedViewsPage"

                InformationPanel {
                    objectName: "sharedViewsInformation"
                    anchors.fill: parent
                    titleText: qsTranslate("TaskPanelInformation", "Information")
                    messages: [qsTranslate("TaskPanelInformation", "Access Shared Views from the Shared panel on the Home tab.")]
                }
            }
        }
    }
}

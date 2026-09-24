// 工程 / 任务 Dock；上方列出工程和 STL，下方展示最新导入零件的任务。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

PanelSurface {
    id: panel

    property bool projectOpen: false
    property string projectName: ""
    property var importedPartNames: []
    property bool importedPartAvailable: importedPartNames.length > 0
    property string importedPartName: ""
    readonly property var importedTaskItems: [
        {
            text: qsTranslate("ImportTask", "Create Mesh..."),
            icon: "mesh"
        },
        {
            text: qsTranslate("ImportTask", "Fill"),
            icon: "geometry"
        },
        {
            text: qsTranslate("ImportTask", "Material Data"),
            icon: "material"
        },
        {
            text: qsTranslate("ImportTask", "Set Injection Locations..."),
            icon: "injection-location"
        },
        {
            text: qsTranslate("ProcessTask", "Process Settings (Default)"),
            icon: "process-settings"
        },
        {
            text: qsTranslate("ImportTask", "Optimization (None)"),
            icon: "optimization"
        },
        {
            text: qsTranslate("UiCommonAnalysis", "Analyze"),
            icon: "analysis-run",
            enabled: false
        },
        {
            text: qsTranslate("ImportTask", "Logs*"),
            icon: "output-copy"
        }
    ]

    signal closeRequested
    signal openProjectRequested
    signal newProjectRequested

    implicitWidth: Theme.leftPanelMinimumWidth

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        PanelTabBar {
            Layout.fillWidth: true
            rightPadding: Theme.paneCloseSize + 2 * Theme.spacingXSmall
            tabs: [qsTranslate("TaskPanelTitle", "Tasks"), qsTranslate("UiCommonNavigation", "Tools"), qsTranslate("UiCommonNavigation", "Shared Views")]
        }

        ColumnLayout {
            id: dockContent
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            ScrollView {
                id: projectTree
                objectName: "projectTreeSection"
                Layout.fillWidth: true
                Layout.preferredHeight: Math.max(0, dockContent.height - (panel.projectOpen ? Theme.borderWidth : 0)) * 0.36
                clip: true
                contentWidth: availableWidth
                ScrollBar.vertical.policy: ScrollBar.AsNeeded

                ColumnLayout {
                    width: projectTree.availableWidth
                    spacing: 0

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0
                        visible: !panel.projectOpen

                        ThemedToolButton {
                            Layout.fillWidth: true
                            text: qsTranslate("IconActionOpenProject", "Open Project")
                            dimText: "…"
                            iconName: "document-open"
                            contentAlignLeft: true
                            contentColor: Theme.colorText
                            contentPadding: Theme.spacingLarge
                            hoverColor: Theme.colorHover
                            onClicked: panel.openProjectRequested()
                        }
                        ThemedToolButton {
                            Layout.fillWidth: true
                            text: qsTranslate("UiCommonNavigation", "New Project")
                            dimText: "…"
                            iconName: "document-new"
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
                        contentAlignLeft: true
                        contentColor: Theme.colorText
                        contentPadding: Theme.spacingLarge
                        hoverColor: Theme.colorHover
                        contentItem: RowLayout {
                            spacing: Theme.spacingMedium
                            ThemedIcon {
                                name: projectEntry.iconName
                                iconSize: projectEntry.iconSize
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

                    Repeater {
                        model: panel.importedPartNames

                        delegate: Rectangle {
                            id: importedPartEntry
                            objectName: "importedPartEntry"
                            required property var modelData
                            required property int index

                            Layout.fillWidth: true
                            Layout.preferredHeight: Theme.controlHeight
                            color: index === panel.importedPartNames.length - 1 ? Theme.colorSelected : "transparent"

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Theme.spacingLarge * 2
                                anchors.rightMargin: Theme.spacingSmall
                                spacing: Theme.spacingSmall

                                ThemedIcon {
                                    name: "mesh"
                                    iconSize: Theme.iconSizeSmall
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
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.borderWidth
                visible: panel.projectOpen
                color: Theme.colorPanelLine
            }

            ScrollView {
                id: studyTaskSection
                objectName: "studyTasksSection"
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredHeight: Math.max(0, dockContent.height - (panel.projectOpen ? Theme.borderWidth : 0)) * 0.64
                clip: true
                contentWidth: availableWidth
                ScrollBar.vertical.policy: ScrollBar.AsNeeded

                ColumnLayout {
                    width: studyTaskSection.availableWidth
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
                                name: "project-file"
                                iconSize: Theme.iconSizeSmall
                            }
                            ThemedLabel {
                                Layout.fillWidth: true
                                text: qsTranslate("ProjectTaskItem", "Study Tasks: %1").arg(panel.importedPartName)
                                textSize: Theme.fontBody
                                elide: Text.ElideRight
                            }
                        }
                    }

                    ThemedToolButton {
                        Layout.fillWidth: true
                        Layout.leftMargin: Theme.spacingMedium
                        text: qsTranslate("ImportTask", "Part (%1)").arg(panel.importedPartName)
                        iconName: "project-file"
                        contentAlignLeft: true
                        contentColor: Theme.colorText
                        contentPadding: Theme.spacingSmall
                        hoverColor: Theme.colorHover
                    }
                    Repeater {
                        model: panel.importedTaskItems

                        delegate: ThemedToolButton {
                            required property var modelData

                            Layout.fillWidth: true
                            Layout.leftMargin: Theme.spacingLarge
                            text: modelData.text
                            iconName: modelData.icon
                            enabled: modelData.enabled !== false
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
}

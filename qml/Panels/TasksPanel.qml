// 任务页签和工程入口；面板只发语义信号，工程命令由页面注入 ViewModel。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

    property bool projectOpen: false
    property string projectName: ""
    property bool importedPartAvailable: false
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

        ColumnLayout {
            id: studyTasks
            Layout.fillWidth: true
            visible: panel.projectOpen && panel.importedPartAvailable
            spacing: 0

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

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

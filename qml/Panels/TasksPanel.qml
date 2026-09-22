// 任务页签和工程入口；面板只发语义信号，工程命令由页面注入 ViewModel。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

    property bool projectOpen: false
    property string projectName: ""

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
            tabs: [qsTranslate("TaskPanelTitle", "Tasks"), qsTr("Tools"), qsTr("Shared Views")]
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
                text: qsTr("New Project")
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

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

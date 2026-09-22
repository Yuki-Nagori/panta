// 任务页签和工程入口；面板只发语义信号，工程命令由页面注入 ViewModel。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

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
            tabs: [qsTr("Task List"), qsTr("Tools"), qsTr("Shared Views")]
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 0

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

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

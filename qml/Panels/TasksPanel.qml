// 任务页签和工程入口；业务命令尚未接入，关闭信号交由页面处理。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

    signal closeRequested

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
                text: qsTr("Open a Project")
                dimText: "…"
                iconName: "document-open"
                contentAlignLeft: true
                contentColor: Theme.colorText
                contentPadding: Theme.spacingLarge
                hoverColor: Theme.colorHover
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
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

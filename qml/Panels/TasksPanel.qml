// 左侧任务面板（复刻件 .tasks-panel）：任务/工具/共享视图页签与任务列表；
// 列表条目暂为视觉骨架，未接业务命令。
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

        // 任务列表（复刻件 .task-list）：条目悬停高亮，弱化后缀跟随主文案。
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

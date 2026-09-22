// 左侧任务面板（复刻件 .tasks-panel）：任务/工具/共享视图页签与任务列表。
// 打开/新建工程暂为视觉骨架；推进修订项承接 ShellViewModel 冒烟命令，
// 经 advanceRevisionTriggered 信号由 App 装配。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

    signal closeRequested
    signal advanceRevisionTriggered

    implicitWidth: Theme.leftPanelMinimumWidth

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        PanelTabBar {
            Layout.fillWidth: true
            tabs: [qsTr("任务"), qsTr("工具"), qsTr("共享视图")]
        }

        // 任务列表（复刻件 .task-list）：条目悬停高亮，弱化后缀跟随主文案。
        ColumnLayout {
            Layout.fillWidth: true
            Layout.topMargin: Theme.spacingMedium
            spacing: 0

            ThemedToolButton {
                Layout.fillWidth: true
                text: qsTr("打开工程")
                dimText: "…"
                iconName: "project-open"
                contentAlignLeft: true
                contentColor: Theme.colorText
                contentPadding: Theme.spacingLarge
                hoverColor: Theme.colorHover
            }
            ThemedToolButton {
                Layout.fillWidth: true
                text: qsTr("新建工程")
                dimText: "…"
                iconName: "project-new"
                contentAlignLeft: true
                contentColor: Theme.colorText
                contentPadding: Theme.spacingLarge
                hoverColor: Theme.colorHover
            }
            ThemedToolButton {
                objectName: "advanceRevisionButton"
                Layout.fillWidth: true
                text: qsTr("推进修订")
                dimText: "…"
                iconName: "animation-preview"
                contentAlignLeft: true
                contentColor: Theme.colorText
                contentPadding: Theme.spacingLarge
                hoverColor: Theme.colorHover
                onClicked: panel.advanceRevisionTriggered()
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

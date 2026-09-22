// Shell 页面装配；标题和错误由 ViewModel 提供，面板关闭状态由页面持有。
// 顶部动作与视图页签尚未接业务命令，窗口控制由系统标题栏承接。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Panta.Bridge

ApplicationWindow {
    minimumWidth: Theme.windowMinimumWidth
    minimumHeight: Theme.windowMinimumHeight
    visible: true
    visibility: Window.Maximized
    title: qsTr("panta")
    color: Theme.colorPanel

    ShellViewModel {
        id: viewModel
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        TopChromePanel {
            Layout.fillWidth: true
            caption: viewModel.caption.length > 0 ? viewModel.caption : "panta 2027"
        }

        RibbonPanel {
            Layout.fillWidth: true
        }

        // 先确定左栏比例宽度，再把剩余区域交给原生视口。
        Item {
            id: workspace

            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                id: leftColumn

                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                // 栏宽取工作区宽的比例，下限 leftPanelMinimumWidth；锚定工作区
                // 宽而非自身宽，避免首选宽与分配宽互相依赖成绑定环。
                width: Math.max(workspace.width * Theme.leftPanelRatio, Theme.leftPanelMinimumWidth)
                spacing: 0

                TasksPanel {
                    id: tasksPanel

                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    onCloseRequested: tasksPanel.visible = false
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Theme.borderWidth
                    color: Theme.colorPanelLine
                }

                OutputPanel {
                    id: outputPanel

                    Layout.fillWidth: true
                    Layout.preferredHeight: leftColumn.height * Theme.outputPanelRatio
                    errorText: viewModel.error
                    onCloseRequested: outputPanel.visible = false
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
                    text: qsTr("Ready")
                    textSize: Theme.fontSmall
                    textColor: Theme.colorTextMuted
                }
            }
        }
    }
}

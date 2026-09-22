// Shell 主窗口（qml.md：组件 PascalCase，id/属性 camelCase；状态用绑定表达）。
// 按 050 复刻件 ai-docs/qml-html/homepage/homepage.html 拼装桌面框架：顶部
// chrome、ribbon 启动区、左侧任务/输出面板、中央 VTK 视口与底部视图页签、
// 状态栏。029 只装配静态骨架：caption 与错误展示保持既有 ViewModel 绑定，
// 其余按钮与页签为视觉参考；窗口控制由系统标题栏承接。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Panta.Bridge

ApplicationWindow {
    id: root

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
            // 重复写入同一 caption 不产生新通知（ShellViewModel 去重，测试覆盖）。
            caption: viewModel.caption.length > 0 ? viewModel.caption : "panta 2027"
        }

        RibbonPanel {
            Layout.fillWidth: true
        }

        // 工作区用 anchors 直接锚定而非嵌套 Layout：嵌套 Layout 的默认最大
        // 尺寸是自身隐式尺寸，fillWidth 列展不开，剩余空间的分派不可预期。
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

        // 状态栏（复刻件 .statusbar）：就绪状态。
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

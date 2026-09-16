// Shell 主窗口（qml.md：组件 PascalCase，id/属性 camelCase；状态用绑定表达）。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Panta.Bridge

ApplicationWindow {
    id: root

    minimumWidth: 640
    minimumHeight: 480
    visible: true
    title: qsTr("panta")
    color: Theme.colorBackground

    ShellViewModel {
        id: viewModel
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium

        Label {
            Layout.fillWidth: true
            text: viewModel.caption.length > 0 ? viewModel.caption : qsTr("panta — 桌面骨架（任务 005）")
            font.pixelSize: Theme.fontTitle
            color: Theme.colorText
        }

        Button {
            text: qsTr("推进修订")
            onClicked: viewModel.tick()
        }

        Label {
            // 重复写入同一 caption 时不产生新通知（ShellViewModel 去重，测试覆盖）。
            text: qsTr("修订计数：") + viewModel.count
            color: Theme.colorTextMuted
        }

        // 未来面板占位：工程树/视口/属性区由后续任务替换（架构：ui-and-bridge）。
        PlaceholderPanel {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        // 错误展示入口：ViewModel 的用户可读摘要（详细诊断走日志，qt.md）。
        Label {
            Layout.fillWidth: true
            visible: viewModel.error.length > 0
            text: viewModel.error
            color: Theme.colorError
            wrapMode: Text.Wrap
        }
    }
}

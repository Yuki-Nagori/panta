// Shell 主窗口（qml.md：组件 PascalCase，id/属性 camelCase；状态用绑定表达）。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Panta.Bridge
import Panta.Visualization

ApplicationWindow {
    id: root

    minimumWidth: Theme.windowMinimumWidth
    minimumHeight: Theme.windowMinimumHeight
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

        ThemedLabel {
            Layout.fillWidth: true
            text: viewModel.caption.length > 0 ? viewModel.caption : qsTr("panta — 桌面骨架（任务 005）")
            textSize: Theme.fontTitle
        }

        ThemedButton {
            text: qsTr("推进修订")
            onClicked: viewModel.tick()
        }

        ThemedLabel {
            // 重复写入同一 caption 时不产生新通知（ShellViewModel 去重，测试覆盖）。
            text: qsTr("修订计数：") + viewModel.count
            textColor: Theme.colorTextMuted
        }

        // CAE 视口（任务 007）：空视口 + 测试图元；工程树/属性区由后续任务替换。
        CaeViewport {
            Layout.fillWidth: true
            Layout.fillHeight: true
            onSceneReady: console.log("CaeViewport 场景就绪")
        }

        // 错误展示入口：ViewModel 的用户可读摘要（详细诊断走日志，qt.md）。
        ThemedLabel {
            Layout.fillWidth: true
            visible: viewModel.error.length > 0
            text: viewModel.error
            textColor: Theme.colorError
            wrapMode: Text.Wrap
        }
    }
}

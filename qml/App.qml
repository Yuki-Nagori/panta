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
    visibility: Window.Maximized
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
            objectName: "shellCaption"
            Layout.fillWidth: true
            text: viewModel.caption.length > 0 ? viewModel.caption : "panta — Desktop Skeleton (Task 005)"
            textSize: Theme.fontTitle
        }

        ThemedButton {
            objectName: "advanceRevisionButton"
            text: "Advance Revision"
            onClicked: viewModel.tick()
        }

        ThemedLabel {
            objectName: "revisionCount"
            // 重复写入同一 caption 时不产生新通知（ShellViewModel 去重，测试覆盖）。
            text: "Revision count: " + viewModel.count
            textColor: Theme.colorTextMuted
        }

        // 任务 007：VTK WebGPU 原生 surface 宿主；VTK 不进入 Qt Quick scenegraph。
        CaeViewport {
            objectName: "caeViewport"
            Layout.fillWidth: true
            Layout.fillHeight: true
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

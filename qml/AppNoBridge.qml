// 不启用 Panta.Bridge 时的最小 Shell；用于资源/依赖裁剪和启动诊断。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root

    minimumWidth: Theme.windowMinimumWidth
    minimumHeight: Theme.windowMinimumHeight
    visible: true
    visibility: Window.Maximized
    title: qsTr("panta")
    color: Theme.colorBackground

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium

        ThemedLabel {
            objectName: "shellCaption"
            Layout.fillWidth: true
            text: qsTr("panta — 最小 Shell（Bridge 已关闭）")
            textSize: Theme.fontTitle
        }

        PlaceholderPanel {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

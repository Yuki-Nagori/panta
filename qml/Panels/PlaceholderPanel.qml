// 无 Bridge 构建中的面板占位，不代表业务功能已经实现。
import QtQuick

PanelSurface {
    ThemedLabel {
        anchors.centerIn: parent
        text: qsTr("Panel placeholder: project tree / viewport / properties (upcoming tasks)")
        textColor: Theme.colorTextMuted
    }
}

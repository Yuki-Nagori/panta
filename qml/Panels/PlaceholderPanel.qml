// 未来面板占位：仅表达布局归属（工程树/视口/属性区），不代表功能完成。
// 同一 QML 模块内的 Theme 与原子组件可直接访问。
import QtQuick

PanelSurface {
    id: panel

    ThemedLabel {
        anchors.centerIn: parent
        text: qsTr("Panel placeholder: project tree / viewport / properties (upcoming tasks)")
        textColor: Theme.colorTextMuted
    }
}

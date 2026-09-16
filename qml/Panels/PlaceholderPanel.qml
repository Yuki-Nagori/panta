// 未来面板占位：仅表达布局归属（工程树/视口/属性区），不代表功能完成。
// 同模块内 Theme 单例可直接访问；显式 import 供 qmllint 的子目录解析。
import QtQuick

Rectangle {
    id: panel

    // 输入属性显式注入（qml.md）；占位面板无输出信号。
    color: Theme.colorPanel
    radius: Theme.spacingSmall

    Text {
        anchors.centerIn: parent
        text: qsTr("面板占位：工程树 / 视口 / 属性区（后续任务）")
        color: Theme.colorTextMuted
        font.pixelSize: Theme.fontBody
    }
}

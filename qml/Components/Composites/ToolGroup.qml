// 标题条工具分组容器（复刻件 .tool-group）：深灰渐变底加 panel-line 包边，
// 经 default 属性承接子按钮行；只负责分组外观，不依赖业务状态。
import QtQuick

Rectangle {
    id: group

    default property alias content: row.data

    implicitHeight: Theme.titlebarHeight
    implicitWidth: row.implicitWidth
    border.width: Theme.borderWidth
    border.color: Theme.colorPanelLine
    gradient: Gradient {
        GradientStop {
            position: 0
            color: Theme.colorToolGroupTop
        }
        GradientStop {
            position: 1
            color: Theme.colorToolGroupBottom
        }
    }

    Row {
        id: row
        objectName: "toolGroupContent"

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingXSmall
    }
}

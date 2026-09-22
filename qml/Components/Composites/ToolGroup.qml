// 标题工具分组：渐变底色与齐边内容行；默认属性接收调用方的按钮。
import QtQuick

Rectangle {
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

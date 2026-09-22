// 扁平工具按钮原子：标题条/工具条/菜单行共用的按钮形态，保留 ToolButton
// 的键盘、焦点与禁用语义；样式默认绑定 Theme token，调用方可显式覆盖，
// 未覆盖属性保持绑定，主题值更新（030）时随之刷新。
import QtQuick
import QtQuick.Controls

ToolButton {
    id: button

    property int controlHeight: Theme.controlHeight
    property int contentPadding: Theme.spacingXSmall
    property color contentColor: Theme.colorIcon
    property color hoverColor: Theme.colorSelected
    property url iconSource: ""
    property int iconSize: Theme.iconSizeDefault
    property bool showCaret: false
    // 追加在主文案后的弱化后缀（复刻件任务列表的 “...” 项）。
    property string dimText: ""

    implicitHeight: controlHeight
    leftPadding: contentPadding
    rightPadding: contentPadding
    spacing: Theme.spacingXSmall

    background: Rectangle {
        implicitWidth: Theme.controlHeight
        color: button.enabled && button.hovered ? button.hoverColor : "transparent"
        radius: Theme.radiusSmall
    }

    contentItem: Row {
        spacing: button.spacing

        Image {
            anchors.verticalCenter: parent.verticalCenter
            visible: button.iconSource !== ""
            source: button.iconSource
            sourceSize: Qt.size(button.iconSize, button.iconSize)
            width: button.iconSize
            height: button.iconSize
            fillMode: Image.PreserveAspectFit
        }
        ThemedLabel {
            anchors.verticalCenter: parent.verticalCenter
            visible: button.text !== ""
            text: button.text
            textColor: button.contentColor
        }
        ThemedLabel {
            anchors.verticalCenter: parent.verticalCenter
            visible: button.dimText !== ""
            text: button.dimText
            textColor: Theme.colorTextMuted
        }
        Image {
            anchors.verticalCenter: parent.verticalCenter
            visible: button.showCaret
            source: Qt.resolvedUrl("../../icons/caret-down.svg")
            sourceSize: Qt.size(Theme.caretWidth, Theme.caretHeight)
            width: Theme.caretWidth
            height: Theme.caretHeight
            fillMode: Image.PreserveAspectFit
        }
    }
}

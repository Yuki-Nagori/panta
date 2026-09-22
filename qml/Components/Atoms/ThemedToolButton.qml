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
    property color borderColor: "transparent"
    property string iconName: ""
    property int iconSize: Theme.iconSizeDefault
    property bool showCaret: false
    // 追加在主文案后的弱化后缀（复刻件任务列表的 “...” 项）。
    property string dimText: ""
    // 宽按钮（列表条目）内容靠左；默认在按钮内水平居中。
    property bool contentAlignLeft: false

    implicitHeight: controlHeight
    leftPadding: contentPadding
    rightPadding: contentPadding
    spacing: Theme.spacingXSmall
    // 禁用态整体弱化，配合 ToolButton 拒绝点击，让禁用原因可理解。
    opacity: enabled ? 1 : Theme.disabledOpacity

    background: Rectangle {
        implicitWidth: Theme.controlHeight
        // highlighted 用 AbstractButton 内建选中态（菜单栏当前项）：白底，
        // 不再响应悬停高亮；visualFocus 使键盘 Tab 焦点获得与悬停一致的反馈。
        color: button.highlighted ? Theme.colorPanel : button.enabled && (button.hovered || button.visualFocus) ? button.hoverColor : "transparent"
        border.width: button.borderColor.a > 0 ? Theme.borderWidth : 0
        border.color: button.borderColor
        radius: Theme.radiusSmall
    }

    contentItem: Row {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: button.contentAlignLeft ? parent.left : undefined
        anchors.horizontalCenter: button.contentAlignLeft ? undefined : parent.horizontalCenter
        spacing: button.spacing

        ThemedIcon {
            anchors.verticalCenter: parent.verticalCenter
            visible: button.iconName !== ""
            name: button.iconName
            iconSize: button.iconSize
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

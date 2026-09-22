// 共用工具按钮：图文、下拉指示、悬停和键盘焦点均由同一内容区域承载。
import QtQuick
import QtQuick.Controls

ToolButton {
    id: button
    hoverEnabled: true

    property int controlHeight: Theme.controlHeight
    property int contentPadding: Theme.spacingXSmall
    property int cornerRadius: Theme.radiusSmall
    property color contentColor: Theme.colorIcon
    property color hoverColor: Theme.colorSelected
    property color borderColor: "transparent"
    property string iconName: ""
    // 纯图标按钮由宿主提供已翻译的名称，供读屏与悬停提示使用。
    property string accessibleName: text
    property int iconSize: Theme.iconSizeDefault
    property bool showCaret: false
    // 主文案后的弱化后缀，例如工程入口的省略号。
    property string dimText: ""
    // 宽按钮（列表条目）内容靠左；默认在按钮内水平居中。
    property bool contentAlignLeft: false

    Accessible.name: accessibleName
    ToolTip.visible: hovered && text === "" && accessibleName !== ""
    ToolTip.text: accessibleName
    ToolTip.delay: Theme.toolTipDelay

    font.pixelSize: Theme.fontBody
    implicitHeight: controlHeight
    leftPadding: contentPadding
    rightPadding: contentPadding
    topPadding: 0
    bottomPadding: 0
    spacing: Theme.spacingXSmall
    // 统一弱化整个内容；禁用事件拦截沿用 ToolButton。
    opacity: enabled ? 1 : Theme.disabledOpacity

    background: Rectangle {
        implicitWidth: Theme.controlHeight
        // highlighted 表示宿主指定的强调态（当前菜单白底），
        // 不再响应悬停高亮；visualFocus 使键盘 Tab 焦点获得与悬停一致的反馈。
        color: button.highlighted ? Theme.colorPanel : button.enabled && (button.hovered || button.visualFocus) ? button.hoverColor : "transparent"
        border.width: button.borderColor.a > 0 ? Theme.borderWidth : 0
        border.color: button.borderColor
        radius: button.cornerRadius
    }

    contentItem: Item {
        implicitWidth: content.implicitWidth
        implicitHeight: content.implicitHeight

        Row {
            id: content
            objectName: "toolButtonContent"
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: button.contentAlignLeft ? parent.left : undefined
            anchors.horizontalCenter: button.contentAlignLeft ? undefined : parent.horizontalCenter
            spacing: button.spacing

            ThemedIcon {
                anchors.verticalCenter: parent.verticalCenter
                visible: button.iconName !== ""
                name: button.iconName
                iconSize: button.iconSize
                color: button.contentColor
            }
            ThemedLabel {
                anchors.verticalCenter: parent.verticalCenter
                visible: button.text !== ""
                text: button.text
                textSize: button.font.pixelSize
                textColor: button.contentColor
            }
            ThemedLabel {
                anchors.verticalCenter: parent.verticalCenter
                visible: button.dimText !== ""
                text: button.dimText
                textSize: button.font.pixelSize
                textColor: Theme.colorTextMuted
            }
            ThemedIcon {
                anchors.verticalCenter: parent.verticalCenter
                visible: button.showCaret
                name: "caret-down"
                iconSize: Theme.iconSizeCaret
                color: button.contentColor
            }
        }
    }
}

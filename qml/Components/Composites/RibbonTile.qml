// Ribbon 启动区磁贴（复刻件 .ribbon-tile）：大图标加小字标签的纵向按钮，
// 白底包边、悬停高亮；保留 ToolButton 的键盘焦点语义。
import QtQuick
import QtQuick.Controls

ToolButton {
    id: tile

    property string iconName: ""
    property int iconSize: Theme.iconSizeRibbon

    leftPadding: Theme.spacingLarge
    rightPadding: Theme.spacingLarge
    topPadding: Theme.spacingSmall
    bottomPadding: Theme.spacingSmall
    spacing: Theme.spacingMedium

    implicitWidth: contentItem.implicitWidth + leftPadding + rightPadding
    implicitHeight: contentItem.implicitHeight + topPadding + bottomPadding

    background: Rectangle {
        color: tile.enabled && tile.hovered ? Theme.colorHover : Theme.colorPanel
        radius: Theme.radiusLarge
        border.width: Theme.borderWidth
        border.color: Theme.colorPanelLine
    }

    contentItem: Column {
        spacing: tile.spacing

        ThemedIcon {
            anchors.horizontalCenter: parent.horizontalCenter
            name: tile.iconName
            iconSize: tile.iconSize
        }
        ThemedLabel {
            anchors.horizontalCenter: parent.horizontalCenter
            text: tile.text
            textSize: Theme.fontSmall
            textColor: Theme.colorText
        }
    }
}

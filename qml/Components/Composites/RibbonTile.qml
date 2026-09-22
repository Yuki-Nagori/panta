// Ribbon 启动区磁贴（复刻件 .ribbon-tile）：大图标加小字标签的纵向按钮，
// 白底包边、悬停高亮；保留 ToolButton 的键盘焦点语义。
import QtQuick
import QtQuick.Controls

ToolButton {
    id: tile
    hoverEnabled: true

    property string iconName: ""
    property int iconSize: Theme.iconSizeRibbon

    opacity: enabled ? 1 : Theme.disabledOpacity
    Accessible.name: text

    leftPadding: Theme.spacingMedium
    rightPadding: Theme.spacingMedium
    topPadding: Theme.spacingXSmall
    bottomPadding: Theme.spacingXSmall
    spacing: Theme.spacingSmall

    implicitWidth: Theme.ribbonTileWidth
    implicitHeight: contentItem.implicitHeight + topPadding + bottomPadding

    background: Rectangle {
        color: tile.enabled && (tile.hovered || tile.visualFocus) ? Theme.colorHover : Theme.colorPanel
        radius: Theme.radiusSmall
        border.width: Theme.borderWidth
        border.color: Theme.colorPanelLine
    }

    contentItem: Item {
        implicitWidth: content.implicitWidth
        implicitHeight: content.implicitHeight

        Column {
            id: content
            objectName: "ribbonTileContent"
            anchors.centerIn: parent
            spacing: tile.spacing

            ThemedIcon {
                anchors.horizontalCenter: parent.horizontalCenter
                name: tile.iconName
                iconSize: tile.iconSize
            }
            ThemedLabel {
                anchors.horizontalCenter: parent.horizontalCenter
                text: tile.text
                textSize: Theme.fontRibbon
                textColor: Theme.colorText
            }
        }
    }
}

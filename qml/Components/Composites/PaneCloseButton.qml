// 面板右上角关闭按钮（复刻件 .pane-close）：悬停红色高亮，输出
// closeRequested 信号由宿主决定隐藏行为；图标为弱化色占位资源。
import QtQuick

ThemedToolButton {
    id: closeButton

    signal closeRequested

    anchors.top: parent.top
    anchors.right: parent.right
    anchors.topMargin: Theme.spacingXSmall
    anchors.rightMargin: Theme.spacingXSmall
    z: 2

    iconName: "pane-close"
    iconSize: Theme.iconSizeSmall
    contentPadding: Theme.spacingXSmall
    hoverColor: Theme.colorCloseHover

    onClicked: closeButton.closeRequested()
}

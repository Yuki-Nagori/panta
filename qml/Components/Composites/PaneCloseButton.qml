// 面板右上角关闭按钮（复刻件 .pane-close）：悬停红色高亮，输出
// closeRequested 信号由宿主决定隐藏行为；图标随悬停/键盘焦点切换为白色。
import QtQuick

ThemedToolButton {
    id: closeButton

    signal closeRequested

    anchors.top: parent.top
    anchors.right: parent.right
    anchors.topMargin: (Theme.panelToolbarHeight - controlHeight) / 2
    anchors.rightMargin: Theme.spacingXSmall
    z: 2

    accessibleName: qsTranslate("IconActionClosePanel", "Close panel")
    iconName: "pane-close"
    width: Theme.paneCloseSize
    controlHeight: Theme.paneCloseSize
    contentColor: hovered || visualFocus ? Theme.colorPanel : Theme.colorTextMuted
    iconSize: Theme.iconSizeSmall
    contentPadding: 0
    hoverColor: Theme.colorCloseHover

    onClicked: closeButton.closeRequested()
}

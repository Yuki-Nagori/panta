// 顶部 chrome 面板（复刻件 .top-chrome）：品牌 logo 列、标题条（快捷图标、
// 激活动图、居中标题、全局搜索与账户簇）与菜单栏。标题经 caption 属性注入；
// 搜索与菜单本任务为视觉骨架，未接业务命令。窗口控制交由系统标题栏承接
// （维护者决策），不复刻最小化/最大化/关闭按钮。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: chrome

    property string caption: ""

    implicitWidth: 800
    implicitHeight: Theme.titlebarHeight + Theme.menubarHeight + Theme.borderWidth
    color: Theme.colorPanel

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // 品牌 logo 列：跨标题条与菜单栏两行（复刻件 .logo-cell）。
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: Theme.logoWidth
            color: Theme.colorBrand

            ThemedLabel {
                anchors.centerIn: parent
                text: "P"
                textSize: Theme.fontTitle
                textColor: Theme.colorPanel
                font.bold: true
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // 标题条行（复刻件 .titlebar）
            RowLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.titlebarHeight
                spacing: Theme.spacingLarge

                ToolGroup {
                    Layout.leftMargin: Theme.spacingXSmall

                    ThemedToolButton {
                        iconName: "document-new"
                        showCaret: true
                    }
                    ThemedToolButton {
                        iconName: "document-open"
                    }
                    ThemedToolButton {
                        iconName: "document-save"
                        showCaret: true
                    }
                    ThemedToolButton {
                        iconName: "edit-undo"
                    }
                    ThemedToolButton {
                        iconName: "edit-redo"
                    }
                    ThemedToolButton {
                        iconName: "document-print"
                    }
                    ThemedToolButton {
                        iconName: "animation-preview"
                        showCaret: true
                    }
                }

                ThemedToolButton {
                    text: qsTr("激活动图(A)")
                    showCaret: true
                    contentColor: Theme.colorText
                    contentPadding: Theme.spacingSmall
                    borderColor: Theme.colorPanelLine
                }

                ThemedIcon {
                    name: "activate-split"
                    iconSize: Theme.iconSizeSmall
                }

                ThemedLabel {
                    objectName: "shellCaption"
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    elide: Text.ElideRight
                    text: chrome.caption
                    textSize: Theme.fontTitle
                    textColor: Theme.colorText
                    font.weight: Font.DemiBold
                }

                ToolGroup {
                    Layout.rightMargin: Theme.spacingXSmall

                    TextField {
                        Layout.preferredWidth: Theme.searchFieldWidth
                        implicitHeight: Theme.searchHeight
                        leftPadding: Theme.spacingXSmall
                        rightPadding: Theme.spacingXSmall
                        placeholderText: qsTr("请输入关键字或短语")
                        color: Theme.colorText
                        font.pixelSize: Theme.fontSmall
                        background: Rectangle {
                            color: Theme.colorPanel
                        }
                    }
                    ThemedToolButton {
                        iconName: "user-account"
                    }
                    ThemedToolButton {
                        text: qsTr("登录")
                        iconName: "user-account"
                        showCaret: true
                    }
                    ThemedToolButton {
                        iconName: "shopping-cart"
                    }
                    ThemedToolButton {
                        iconName: "help-browser"
                        showCaret: true
                    }
                }
            }

            // 菜单栏行（复刻件 .menubar）：深灰底，选中项白底高亮。
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: Theme.colorMenubar

                Row {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: Theme.spacingTiny

                    ThemedToolButton {
                        text: qsTr("开始并学习")
                        highlighted: true
                        contentColor: Theme.colorText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                        font.weight: Font.DemiBold
                    }
                    ThemedToolButton {
                        text: qsTr("社区")
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                    }
                    ThemedToolButton {
                        text: qsTr("工具")
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                    }
                    ThemedToolButton {
                        text: qsTr("查看")
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                    }

                    Item {
                        width: Theme.spacingMedium
                    }
                    ThemedToolButton {
                        iconName: "menubar-globe"
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                    }
                    ThemedToolButton {
                        iconName: "menubar-chevron"
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                    }
                }
            }
        }
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: Theme.borderWidth
        color: Theme.colorChromeLine
    }
}

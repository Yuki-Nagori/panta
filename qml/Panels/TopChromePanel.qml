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

            // 小窗口横向滚动标题工具区，避免压缩按钮或遮住搜索/账户操作。
            Flickable {
                id: titleStrip
                objectName: "titleStrip"
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.titlebarHeight
                contentWidth: titleContent.width
                contentHeight: height
                flickableDirection: Flickable.HorizontalFlick
                boundsBehavior: Flickable.StopAtBounds
                clip: true

                ScrollBar.horizontal: ScrollBar {
                    policy: ScrollBar.AsNeeded
                }

                // 聚焦与窗口收窄都会改变可见区域；布局完成后再滚到焦点控件。
                function ensureFocusedVisible() {
                    const hostWindow = chrome.Window.window;
                    const focused = hostWindow ? hostWindow.activeFocusItem : null;
                    let ancestor = focused;
                    while (ancestor && ancestor !== titleContent)
                        ancestor = ancestor.parent;
                    if (!ancestor || !focused)
                        return;
                    const point = focused.mapToItem(titleContent, 0, 0);
                    const desired = point.x < contentX ? point.x : Math.max(contentX, point.x + focused.width - width);
                    contentX = Math.max(0, Math.min(desired, contentWidth - width));
                }
                onWidthChanged: Qt.callLater(ensureFocusedVisible)
                Connections {
                    target: chrome.Window.window
                    function onActiveFocusItemChanged() {
                        Qt.callLater(titleStrip.ensureFocusedVisible);
                    }
                }

                RowLayout {
                    id: titleContent
                    width: Math.max(titleStrip.width, Theme.titlebarMinimumContentWidth)
                    height: titleStrip.height
                    spacing: Theme.spacingMedium

                    ToolGroup {
                        objectName: "titleQuickActions"
                        ThemedToolButton {
                            iconName: "document-new"
                            accessibleName: qsTranslate("IconActionNewDocument", "New document")
                            showCaret: true
                        }
                        ThemedToolButton {
                            iconName: "document-open"
                            accessibleName: qsTranslate("IconActionOpenDocument", "Open document")
                        }
                        ThemedToolButton {
                            iconName: "document-save"
                            accessibleName: qsTranslate("IconActionSaveDocument", "Save document")
                            showCaret: true
                        }
                        ThemedToolButton {
                            iconName: "edit-undo"
                            accessibleName: qsTranslate("IconActionUndo", "Undo")
                        }
                        ThemedToolButton {
                            iconName: "edit-redo"
                            accessibleName: qsTranslate("IconActionRedo", "Redo")
                        }
                        ThemedToolButton {
                            iconName: "document-print"
                            accessibleName: qsTranslate("IconActionPrint", "Print")
                        }
                        ThemedToolButton {
                            iconName: "animation-preview"
                            accessibleName: qsTranslate("IconActionPreviewAnimation", "Preview animation")
                            showCaret: true
                        }
                        ThemedToolButton {
                            text: qsTr("Activate Animation (A)")
                            showCaret: true
                            contentColor: Theme.colorText
                            contentPadding: Theme.spacingSmall
                            borderColor: Theme.colorPanelLine
                        }
                        ThemedIcon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: "activate-split"
                            iconSize: Theme.iconSizeSmall
                            color: Theme.colorIcon
                        }
                    }
                    ThemedLabel {
                        objectName: "shellCaption"
                        Layout.fillWidth: true
                        Layout.minimumWidth: Theme.spacingLarge
                        horizontalAlignment: Text.AlignHCenter
                        elide: Text.ElideRight
                        text: chrome.caption
                        textSize: Theme.fontTitle
                        textColor: Theme.colorText
                        font.weight: Font.DemiBold
                    }
                    ToolGroup {
                        objectName: "titleSearchGroup"
                        Row {
                            spacing: Theme.spacingXSmall
                            Item {
                                anchors.verticalCenter: parent.verticalCenter
                                width: Theme.iconSizeDefault
                                height: Theme.searchHeight
                                ThemedIcon {
                                    anchors.centerIn: parent
                                    name: "caret-right"
                                    iconSize: Theme.iconSizeCaret
                                }
                            }
                            TextField {
                                id: searchField
                                anchors.verticalCenter: parent.verticalCenter
                                objectName: "globalSearch"
                                width: Theme.searchFieldWidth
                                height: Theme.searchHeight
                                topPadding: 0
                                bottomPadding: 0
                                leftPadding: Theme.spacingSmall
                                rightPadding: Theme.spacingSmall
                                placeholderText: qsTr("Enter a keyword or phrase")
                                Accessible.name: placeholderText
                                color: Theme.colorText
                                font.pixelSize: Theme.fontSmall
                                background: Rectangle {
                                    color: Theme.colorPanel
                                    border.width: Theme.borderWidth
                                    border.color: searchField.activeFocus ? Theme.colorIcon : Theme.colorPanelLine
                                }
                            }
                            ThemedToolButton {
                                objectName: "searchButton"
                                iconName: "binoculars"
                                accessibleName: qsTranslate("IconActionSearch", "Search")
                            }
                        }
                        ThemedToolButton {
                            text: qsTr("Sign in")
                            iconName: "user-account"
                            showCaret: true
                        }
                        ThemedToolButton {
                            iconName: "shopping-cart"
                            accessibleName: qsTranslate("IconActionShoppingCart", "Shopping cart")
                        }
                        ThemedToolButton {
                            iconName: "help-browser"
                            accessibleName: qsTranslate("IconActionHelp", "Help")
                            showCaret: true
                        }
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
                        text: qsTr("Start and Learn")
                        cornerRadius: 0
                        highlighted: true
                        contentColor: Theme.colorText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                        font.weight: Font.DemiBold
                    }
                    ThemedToolButton {
                        text: qsTr("Community")
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                    }
                    ThemedToolButton {
                        text: qsTr("Tools")
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                    }
                    ThemedToolButton {
                        text: qsTr("View")
                        contentColor: Theme.colorMenubarText
                        hoverColor: Theme.colorMenubarHover
                        contentPadding: Theme.spacingLarge
                    }

                    Item {
                        width: Theme.spacingMedium
                    }
                    ThemedToolButton {
                        iconName: "menubar-globe"
                        showCaret: true
                        accessibleName: qsTranslate("IconActionLanguage", "Language")
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

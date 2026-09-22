// 品牌、快捷工具、标题、搜索和菜单；caption 由宿主注入，动作尚未接业务服务。
// 原生窗口控制仍由系统标题栏承接。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQml

Rectangle {
    id: chrome

    property string caption: ""

    implicitWidth: 800
    implicitHeight: Theme.titlebarHeight + Theme.menubarHeight + Theme.borderWidth
    color: Theme.colorPanel

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // 品牌区域跨标题工具条与菜单栏。
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
                // RowLayout updates child positions after its width binding settles. Queue a
                // second pass so a newly wider quick-action group cannot leave a focused search
                // field beyond the visible strip.
                function scheduleEnsureFocusedVisible() {
                    Qt.callLater(function() { Qt.callLater(ensureFocusedVisible); });
                }
                onWidthChanged: scheduleEnsureFocusedVisible()
                onContentWidthChanged: scheduleEnsureFocusedVisible()
                Connections {
                    target: chrome.Window.window
                    function onActiveFocusItemChanged() {
                        titleStrip.scheduleEnsureFocusedVisible();
                    }
                }

                RowLayout {
                    id: titleContent
                    // 标题可省略，工具分组不可压缩；翻译或字号增长时扩展滚动范围。
                    width: Math.max(titleStrip.width, Theme.titlebarMinimumContentWidth, quickActions.implicitWidth + searchActions.implicitWidth + Theme.spacingLarge + 2 * spacing)
                    height: titleStrip.height
                    spacing: Theme.spacingMedium

                    ToolGroup {
                        id: quickActions
                        objectName: "titleQuickActions"
                        Layout.minimumWidth: implicitWidth
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
                        id: searchActions
                        objectName: "titleSearchGroup"
                        Layout.minimumWidth: implicitWidth
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

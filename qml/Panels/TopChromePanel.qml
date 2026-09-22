// 品牌、快捷工具、标题、搜索和菜单；caption 由宿主注入，动作尚未接业务服务。
// 原生窗口控制仍由系统标题栏承接。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: chrome

    property string caption: ""
    property bool projectOpen: false

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
            HorizontalToolStrip {
                id: titleStrip
                objectName: "titleStrip"
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.titlebarHeight
                contentRoot: titleContent

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

                HorizontalToolStrip {
                    id: menuStrip
                    objectName: "menuStrip"
                    anchors.fill: parent
                    contentRoot: menuContent

                    Row {
                        id: menuContent
                        height: menuStrip.height

                        Repeater {
                            model: chrome.projectOpen ? [
                                {
                                    key: "home",
                                    label: qsTranslate("ShellMenuHome", "Home")
                                },
                                {
                                    key: "tools",
                                    label: qsTranslate("ShellMenuTools", "Tools")
                                },
                                {
                                    key: "view",
                                    label: qsTranslate("ShellMenuView", "View")
                                },
                                {
                                    key: "geometry",
                                    label: qsTranslate("ShellMenuGeometry", "Geometry")
                                },
                                {
                                    key: "mesh",
                                    label: qsTranslate("ShellMenuMesh", "Mesh")
                                },
                                {
                                    key: "boundary",
                                    label: qsTranslate("ShellMenuBoundary", "Boundary Conditions")
                                },
                                {
                                    key: "optimization",
                                    label: qsTranslate("ShellMenuOptimization", "Optimization")
                                },
                                {
                                    key: "results",
                                    label: qsTranslate("ShellMenuResults", "Results")
                                },
                                {
                                    key: "reports",
                                    label: qsTranslate("ShellMenuReports", "Reports")
                                },
                                {
                                    key: "start-learn",
                                    label: qsTranslate("ShellMenuStartLearn", "Start & Learn")
                                },
                                {
                                    key: "community",
                                    label: qsTranslate("ShellMenuCommunity", "Community")
                                }
                            ] : [
                                {
                                    key: "start-learn",
                                    label: qsTranslate("ShellMenuStartLearn", "Start & Learn")
                                },
                                {
                                    key: "community",
                                    label: qsTranslate("ShellMenuCommunity", "Community")
                                },
                                {
                                    key: "tools",
                                    label: qsTranslate("ShellMenuTools", "Tools")
                                },
                                {
                                    key: "view",
                                    label: qsTranslate("ShellMenuView", "View")
                                }
                            ]
                            delegate: ThemedToolButton {
                                required property int index
                                required property var modelData
                                objectName: "menu-" + modelData.key
                                text: modelData.label
                                controlHeight: Theme.menubarHeight
                                cornerRadius: 0
                                highlighted: index === 0
                                contentColor: highlighted ? Theme.colorText : Theme.colorMenubarText
                                hoverColor: Theme.colorMenubarHover
                                contentPadding: Theme.spacingLarge
                                font.weight: highlighted ? Font.DemiBold : Font.Normal
                            }
                        }
                        ThemedToolButton {
                            objectName: "languageButton"
                            iconName: "menubar-globe"
                            showCaret: true
                            accessibleName: qsTranslate("IconActionLanguage", "Language")
                            controlHeight: Theme.menubarHeight
                            cornerRadius: 0
                            contentColor: Theme.colorMenubarText
                            hoverColor: Theme.colorMenubarHover
                            contentPadding: Theme.spacingLarge
                        }
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

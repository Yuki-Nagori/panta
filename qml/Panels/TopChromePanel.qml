// 品牌、快捷工具、标题、搜索和菜单；宿主注入标题与页签状态，导航只发语义请求。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: chrome

    property string caption: ""
    property bool projectOpen: false
    property string activeRibbonTab: "start-learn"
    property real nativeControlsLeftInset: 0
    property real nativeControlsRightInset: 0
    property Window hostWindow
    property var recentProjects: []
    property bool projectCommandsEnabled: true

    // 面板仅报告稳定菜单 key，由宿主决定哪些入口可导航。
    signal menuRequested(string key)
    signal newProjectRequested
    signal openProjectRequested
    signal recentProjectRequested(string path)
    signal settingsRequested
    signal exitRequested

    implicitWidth: Theme.chromeImplicitWidth
    implicitHeight: Theme.titlebarHeight + Theme.menubarHeight + Theme.borderWidth
    color: Theme.colorChrome

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // macOS 使用系统交通灯，不再重复显示应用内品牌块。
        Rectangle {
            objectName: "titleBrandTile"
            visible: Qt.platform.os !== "osx"
            Layout.fillHeight: true
            Layout.preferredWidth: Theme.logoWidth
            color: Theme.colorBrand

            ToolButton {
                id: applicationMenuButton
                objectName: "applicationMenuButton"
                anchors.fill: parent
                Accessible.name: qsTranslate("TopChromePanel", "Application menu")
                display: AbstractButton.TextOnly
                text: "P"
                font.pixelSize: Theme.fontTitle
                font.bold: true
                palette.buttonText: Theme.colorPanel
                hoverEnabled: true
                background: Rectangle {
                    color: applicationBackstage.visible || applicationMenuButton.down || applicationMenuButton.hovered ? Qt.darker(Theme.colorBrand, 1.2) : Theme.colorBrand
                    border.width: applicationMenuButton.visualFocus ? Theme.focusBorderWidth : 0
                    border.color: Theme.colorFocus
                }
                onClicked: applicationBackstage.visible ? applicationBackstage.close() : applicationBackstage.openAt(applicationMenuButton)
            }

            ApplicationBackstage {
                id: applicationBackstage
                recentProjects: chrome.recentProjects
                projectCommandsEnabled: chrome.projectCommandsEnabled
                onNewProjectRequested: chrome.newProjectRequested()
                onOpenProjectRequested: chrome.openProjectRequested()
                onRecentProjectRequested: path => chrome.recentProjectRequested(path)
                onSettingsRequested: chrome.settingsRequested()
                onExitRequested: chrome.exitRequested()
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
                Layout.leftMargin: chrome.nativeControlsLeftInset
                Layout.rightMargin: chrome.nativeControlsRightInset
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
                        ChromeToolButton {
                            iconName: "new"
                            preserveIconColors: true
                            accessibleName: qsTranslate("IconActionNewDocument", "New document")
                            showCaret: true
                        }
                        ChromeToolButton {
                            iconName: "open"
                            preserveIconColors: true
                            accessibleName: qsTranslate("IconActionOpenDocument", "Open document")
                        }
                        ChromeToolButton {
                            iconName: "save"
                            preserveIconColors: true
                            accessibleName: qsTranslate("IconActionSaveDocument", "Save document")
                            showCaret: true
                        }
                        ChromeToolButton {
                            iconName: "undo"
                            accessibleName: qsTranslate("IconActionUndo", "Undo")
                        }
                        ChromeToolButton {
                            iconName: "redo"
                            accessibleName: qsTranslate("IconActionRedo", "Redo")
                        }
                        ChromeToolButton {
                            iconName: "print"
                            accessibleName: qsTranslate("IconActionPrint", "Print")
                        }
                        ChromeToolButton {
                            iconName: "preview"
                            accessibleName: qsTranslate("IconActionPreviewAnimation", "Preview animation")
                            showCaret: true
                        }
                        ChromeToolButton {
                            text: qsTr("Activate Animation (A)")
                            showCaret: true
                            contentColor: Theme.colorText
                            contentPadding: Theme.spacingSmall
                            borderColor: Theme.colorPanelLine
                        }
                        ThemedIcon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: "split"
                            iconSize: Theme.iconSizeSmall
                            color: Theme.colorIcon
                        }
                    }
                    Item {
                        id: chromeCaptionArea
                        objectName: "shellCaptionDragArea"
                        Layout.fillWidth: true
                        Layout.minimumWidth: Theme.spacingLarge
                        implicitHeight: Theme.titlebarHeight

                        ThemedLabel {
                            objectName: "shellCaption"
                            anchors.fill: parent
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                            elide: Text.ElideRight
                            text: chrome.caption
                            visible: text.length > 0
                            textSize: Theme.fontTitle
                            textColor: Theme.colorText
                            font.weight: Font.DemiBold
                        }

                        // 扩展客户区覆盖了系统标题栏；窗口移动交给各平台的系统窗口管理器。
                        DragHandler {
                            parent: chromeCaptionArea
                            target: null
                            acceptedButtons: Qt.LeftButton
                            onActiveChanged: function () {
                                if (active && chrome.hostWindow && !chrome.hostWindow.startSystemMove())
                                    console.warn("The platform window system rejected the title-bar move request.");
                            }
                        }

                        TapHandler {
                            parent: chromeCaptionArea
                            acceptedButtons: Qt.LeftButton
                            exclusiveSignals: TapHandler.DoubleTap
                            onDoubleTapped: (eventPoint, button) => {
                                if (button === Qt.LeftButton)
                                    WindowChromeMetrics.performTitlebarDoubleClickAction();
                            }
                        }
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
                                    name: "right"
                                    iconSize: Theme.iconSizeCompact
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
                            ChromeToolButton {
                                objectName: "searchButton"
                                iconName: "search"
                                accessibleName: qsTranslate("IconActionSearch", "Search")
                            }
                        }
                        ChromeToolButton {
                            text: qsTr("Sign in")
                            iconName: "account"
                            showCaret: true
                        }
                        ChromeToolButton {
                            iconName: "cart"
                            accessibleName: qsTranslate("IconActionShoppingCart", "Shopping cart")
                        }
                        ChromeToolButton {
                            iconName: "help"
                            accessibleName: qsTranslate("UiCommonHelp", "Help")
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
                                    label: qsTranslate("UiCommonNavigation", "Home")
                                },
                                {
                                    key: "tools",
                                    label: qsTranslate("UiCommonNavigation", "Tools")
                                },
                                {
                                    key: "view",
                                    label: qsTranslate("ShellMenuView", "View")
                                },
                                {
                                    key: "geometry",
                                    label: qsTranslate("UiCommonModeling", "Geometry")
                                },
                                {
                                    key: "mesh",
                                    label: qsTranslate("UiCommonModeling", "Mesh")
                                },
                                {
                                    key: "boundary",
                                    label: qsTranslate("ShellMenuBoundary", "Boundary Conditions")
                                },
                                {
                                    key: "optimization",
                                    label: qsTranslate("UiCommonModeling", "Optimization")
                                },
                                {
                                    key: "results",
                                    label: qsTranslate("UiCommonResults", "Results")
                                },
                                {
                                    key: "reports",
                                    label: qsTranslate("UiCommonReports", "Reports")
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
                                    label: qsTranslate("UiCommonNavigation", "Tools")
                                },
                                {
                                    key: "view",
                                    label: qsTranslate("ShellMenuView", "View")
                                }
                            ]
                            delegate: MenuTabButton {
                                required property var modelData
                                objectName: "menu-" + modelData.key
                                text: modelData.label
                                highlighted: modelData.key === chrome.activeRibbonTab
                                onClicked: chrome.menuRequested(modelData.key)
                            }
                        }
                        ThemedToolButton {
                            objectName: "languageButton"
                            iconName: "globe"
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

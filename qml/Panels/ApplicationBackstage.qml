// Windows/Linux 的应用后台面板；只发出菜单意图，业务命令由宿主承接。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Popup {
    id: backstage

    objectName: "applicationBackstage"
    property var recentProjects: []
    property bool projectCommandsEnabled: true
    property string searchText: ""
    property bool sortByName: false

    signal newProjectRequested
    signal openProjectRequested
    signal recentProjectRequested(string path)
    signal settingsRequested
    signal exitRequested

    readonly property var filteredRecentProjects: {
        const query = searchText.trim().toLocaleLowerCase();
        const paths = Array.from(recentProjects || []);
        const filtered = query.length === 0 ? paths : paths.filter(path => path.toLocaleLowerCase().includes(query));
        return sortByName ? filtered.sort((left, right) => left.localeCompare(right)) : filtered;
    }

    function openAt(anchorItem) {
        const position = anchorItem.mapToItem(parent, 0, 0);
        x = position.x;
        y = position.y;
        open();
    }

    parent: Overlay.overlay
    x: 0
    y: 0
    width: Overlay.overlay && Overlay.overlay.width > 0 ? Math.max(1, Math.min(Theme.applicationMenuWidth, Overlay.overlay.width - x)) : Theme.applicationMenuWidth
    height: Overlay.overlay && Overlay.overlay.height > 0 ? Math.max(1, Math.min(Theme.applicationMenuHeight, Overlay.overlay.height - y)) : Theme.applicationMenuHeight
    padding: 0
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    onOpened: {
        if (projectCommandsEnabled)
            backstageNewCommand.forceActiveFocus(Qt.PopupFocusReason);
        else
            backstageSettingsButton.forceActiveFocus(Qt.PopupFocusReason);
    }

    background: Rectangle {
        color: Theme.colorPanel
        border.width: Theme.borderWidth
        border.color: Theme.colorPanelLine
    }

    contentItem: RowLayout {
        spacing: 0

        Rectangle {
            Layout.preferredWidth: Theme.applicationMenuSidebarWidth
            Layout.fillHeight: true
            color: Theme.colorChrome

            ColumnLayout {
                anchors.fill: parent
                spacing: Theme.spacingTiny

                Rectangle {
                    Layout.alignment: Qt.AlignLeft
                    Layout.preferredWidth: Theme.logoWidth
                    Layout.preferredHeight: Theme.titlebarHeight + Theme.menubarHeight
                    color: Theme.colorBrand

                    ToolButton {
                        id: backstageBrandButton
                        anchors.fill: parent
                        objectName: "backstageBrandButton"
                        text: "P"
                        Accessible.name: qsTranslate("ApplicationBackstage", "Close application menu")
                        palette.buttonText: Theme.colorPanel
                        font.pixelSize: Theme.fontTitle + Theme.spacingMedium
                        font.bold: true
                        hoverEnabled: true
                        background: Rectangle {
                            color: Qt.darker(Theme.colorBrand, 1.2)
                            border.width: backstageBrandButton.visualFocus ? Theme.focusBorderWidth : 0
                            border.color: Theme.colorFocus
                        }
                        onClicked: backstage.close()
                    }
                }

                Item {
                    Layout.preferredHeight: Theme.spacingSmall
                }

                ApplicationBackstageCommand {
                    id: backstageNewCommand
                    Layout.fillWidth: true
                    objectName: "backstageNewProject"
                    text: qsTranslate("ApplicationBackstage", "New")
                    iconName: "new"
                    preserveIconColors: true
                    enabled: backstage.projectCommandsEnabled
                    onClicked: {
                        backstage.close();
                        backstage.newProjectRequested();
                    }
                }
                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    objectName: "backstageOpenProject"
                    text: qsTranslate("ApplicationBackstage", "Open")
                    iconName: "open"
                    preserveIconColors: true
                    enabled: backstage.projectCommandsEnabled
                    onClicked: {
                        backstage.close();
                        backstage.openProjectRequested();
                    }
                }
                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    text: qsTranslate("ApplicationBackstage", "Save")
                    iconName: "save"
                    preserveIconColors: true
                    enabled: false
                }
                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    text: qsTranslate("ApplicationBackstage", "Export")
                    iconName: "export"
                    enabled: false
                }
                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    text: qsTranslate("ApplicationBackstage", "Publish")
                    iconName: "export"
                    enabled: false
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.spacingSmall
                    Layout.rightMargin: Theme.spacingSmall
                    Layout.preferredHeight: Theme.borderWidth
                    color: Theme.colorPanelLine
                }

                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    text: qsTranslate("ApplicationBackstage", "Print")
                    iconName: "print"
                    enabled: false
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.spacingSmall
                    Layout.rightMargin: Theme.spacingSmall
                    Layout.preferredHeight: Theme.borderWidth
                    color: Theme.colorPanelLine
                }

                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    text: qsTranslate("ApplicationBackstage", "Project")
                    iconName: "ribbon-project"
                    preserveIconColors: true
                    enabled: false
                }
                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    objectName: "backstageStudyProperties"
                    text: qsTranslate("ApplicationBackstage", "Study Properties")
                    iconName: "ribbon-analysis-sequence"
                    preserveIconColors: true
                    enabled: false
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.spacingSmall
                    Layout.rightMargin: Theme.spacingSmall
                    Layout.preferredHeight: Theme.borderWidth
                    color: Theme.colorPanelLine
                }

                ApplicationBackstageCommand {
                    Layout.fillWidth: true
                    objectName: "backstageCloseCommand"
                    text: qsTranslate("ApplicationBackstage", "Close")
                    iconName: "close"
                    onClicked: {}
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: Theme.spacingLarge
            spacing: Theme.spacingMedium

            RowLayout {
                Layout.fillWidth: true
                Item {
                    Layout.fillWidth: true
                }
                ThemedTextField {
                    id: recentSearch
                    objectName: "recentProjectsSearch"
                    Layout.preferredWidth: Math.min(Theme.searchFieldWidth, backstage.width - Theme.applicationMenuSidebarWidth - 2 * Theme.spacingLarge)
                    Layout.preferredHeight: Theme.searchHeight + 2 * Theme.spacingSmall
                    placeholderText: qsTranslate("ApplicationBackstage", "Search recent projects")
                    Accessible.name: placeholderText
                    onTextChanged: backstage.searchText = text
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingSmall
                ThemedIcon {
                    Layout.preferredWidth: Theme.iconSizeSmall
                    Layout.preferredHeight: Theme.iconSizeSmall
                    name: "history"
                    color: Theme.colorIcon
                }
                ThemedLabel {
                    objectName: "recentProjectsHeading"
                    Layout.fillWidth: true
                    text: qsTranslate("ApplicationBackstage", "Recent Projects")
                    textSize: Theme.fontTitle + Theme.spacingMedium
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingSmall
                ThemedLabel {
                    text: qsTranslate("ApplicationBackstage", "Sort by")
                    textSize: Theme.fontBody
                }
                ThemedComboBox {
                    objectName: "recentProjectsSort"
                    Layout.preferredHeight: Theme.controlHeight + Theme.spacingXSmall
                    model: [qsTranslate("ApplicationBackstage", "Last opened"), qsTranslate("ApplicationBackstage", "Name")]
                    onCurrentIndexChanged: backstage.sortByName = currentIndex === 1
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.borderWidth
                color: Theme.colorPanelLine
            }

            ListView {
                id: recentProjectsList
                objectName: "recentProjectsList"
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: backstage.filteredRecentProjects
                delegate: ItemDelegate {
                    id: recentProjectDelegate
                    objectName: "recentProjectDelegate"
                    required property string modelData
                    width: recentProjectsList.width
                    height: Theme.applicationMenuCommandHeight
                    text: modelData.split(/[\\/]/).pop()
                    Accessible.name: modelData
                    contentItem: ThemedLabel {
                        leftPadding: Theme.spacingSmall
                        text: recentProjectDelegate.text
                        textSize: Theme.fontBody
                    }
                    background: Rectangle {
                        color: recentProjectDelegate.hovered || recentProjectDelegate.visualFocus ? Theme.colorSelected : Theme.colorTransparent
                    }
                    onClicked: {
                        backstage.close();
                        backstage.recentProjectRequested(recentProjectDelegate.modelData);
                    }
                }

                ThemedLabel {
                    anchors.centerIn: parent
                    visible: recentProjectsList.count === 0
                    text: qsTranslate("ApplicationBackstage", "Projects you open will appear here.")
                    textColor: Theme.colorTextMuted
                    horizontalAlignment: Text.AlignHCenter
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignRight
                spacing: Theme.spacingSmall
                Item {
                    Layout.fillWidth: true
                }
                ThemedButton {
                    id: backstageSettingsButton
                    objectName: "backstageSettingsButton"
                    text: qsTranslate("DialogAction", "Settings")
                    onClicked: {
                        backstage.close();
                        backstage.settingsRequested();
                    }
                }
                ThemedButton {
                    objectName: "backstageExitButton"
                    text: qsTranslate("ApplicationBackstage", "Exit panta")
                    implicitWidth: Theme.dialogActionWidth + Theme.spacingLarge
                    onClicked: {
                        backstage.close();
                        backstage.exitRequested();
                    }
                }
            }
        }
    }
}

// 输出面板（复刻件 .output-panel）：输出工具条与消息体。可观察错误经
// errorText 注入显示（qt.md：可观察错误进 UI，详细诊断走日志）；工具条
// 按钮本任务为视觉骨架，未接业务命令。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

    property string errorText: ""

    signal closeRequested

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // 输出工具条（复刻件 .output-toolbar）
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.controlHeight
            color: Theme.colorChrome

            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingTiny

                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-new"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-open"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-save"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-check"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-wizard"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-clear"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-copy"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-image"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-export"
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-delete"
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.borderWidth
            color: Theme.colorPanelLine
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ThemedLabel {
                anchors.fill: parent
                anchors.margins: Theme.spacingMedium
                visible: panel.errorText.length > 0
                text: panel.errorText
                textSize: Theme.fontBody
                textColor: Theme.colorError
                wrapMode: Text.Wrap
                elide: Text.ElideRight
            }
        }
    }
}

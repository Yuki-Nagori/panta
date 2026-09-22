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
            Layout.preferredHeight: Theme.panelToolbarHeight
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
                    accessibleName: qsTranslate("IconActionNewOutput", "New output")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "document-open"
                    accessibleName: qsTranslate("IconActionOpenOutput", "Open output")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "document-save"
                    accessibleName: qsTranslate("IconActionSaveOutput", "Save output")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-check"
                    accessibleName: qsTranslate("IconActionCheckOutput", "Check output")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-wizard"
                    accessibleName: qsTranslate("IconActionOutputWizard", "Output wizard")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "pane-close"
                    accessibleName: qsTranslate("IconActionClearOutput", "Clear output")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-copy"
                    accessibleName: qsTranslate("IconActionCopyOutput", "Copy output")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-image"
                    accessibleName: qsTranslate("IconActionOutputImage", "Output image")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-export"
                    accessibleName: qsTranslate("IconActionExportOutput", "Export output")
                }
                ThemedToolButton {
                    width: Theme.toolbarButtonWidth
                    controlHeight: Theme.toolbarButtonHeight
                    contentPadding: 0
                    iconSize: Theme.iconSizeSmall
                    iconName: "output-delete"
                    accessibleName: qsTranslate("IconActionDeleteOutput", "Delete output")
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

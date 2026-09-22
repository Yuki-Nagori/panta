// 输出工具条与错误消息面板；错误由宿主注入，工具动作尚未接业务服务。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

    property string errorText: ""

    signal closeRequested

    component OutputAction: ThemedToolButton {
        width: Theme.toolbarButtonWidth
        controlHeight: Theme.toolbarButtonHeight
        contentPadding: 0
        iconSize: Theme.iconSizeSmall
    }

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.panelToolbarHeight
            color: Theme.colorChrome

            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingTiny

                OutputAction {
                    iconName: "output-new"
                    accessibleName: qsTranslate("IconActionNewOutput", "New output")
                }
                OutputAction {
                    iconName: "document-open"
                    accessibleName: qsTranslate("IconActionOpenOutput", "Open output")
                }
                OutputAction {
                    iconName: "document-save"
                    accessibleName: qsTranslate("IconActionSaveOutput", "Save output")
                }
                OutputAction {
                    iconName: "output-check"
                    accessibleName: qsTranslate("IconActionCheckOutput", "Check output")
                }
                OutputAction {
                    iconName: "output-wizard"
                    accessibleName: qsTranslate("IconActionOutputWizard", "Output wizard")
                }
                OutputAction {
                    iconName: "pane-close"
                    accessibleName: qsTranslate("IconActionClearOutput", "Clear output")
                }
                OutputAction {
                    iconName: "output-copy"
                    accessibleName: qsTranslate("IconActionCopyOutput", "Copy output")
                }
                OutputAction {
                    iconName: "output-image"
                    accessibleName: qsTranslate("IconActionOutputImage", "Output image")
                }
                OutputAction {
                    iconName: "output-export"
                    accessibleName: qsTranslate("IconActionExportOutput", "Export output")
                }
                OutputAction {
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

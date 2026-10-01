// 网格工具的纯界面；参数和动作尚未接入网格服务。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ScrollView {
    id: meshTool
    objectName: "meshToolPanel"

    clip: true
    contentWidth: availableWidth
    ScrollBar.vertical.policy: ScrollBar.AlwaysOff
    ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

    ColumnLayout {
        width: meshTool.availableWidth
        spacing: Theme.spacingMedium

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            Layout.rightMargin: Theme.spacingMedium
            Layout.topMargin: Theme.spacingMedium
            spacing: Theme.spacingSmall

            ThemedComboBox {
                id: operationSelect

                font.pixelSize: Theme.fontBody
                objectName: "meshOperationSelect"
                Layout.fillWidth: true

                model: [qsTranslate("MeshTool", "Generate Mesh")]
                Accessible.name: qsTranslate("MeshTool", "Mesh operation")
                rightPadding: Theme.iconSizeSmall + shortcutHint.implicitWidth + 3 * Theme.spacingSmall

                ThemedLabel {
                    id: shortcutHint
                    anchors.right: operationSelect.indicator.left
                    anchors.rightMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    text: "F2"
                    textSize: Theme.fontSmall
                    textColor: Theme.colorTextMuted
                    leftPadding: Theme.spacingXSmall
                    rightPadding: Theme.spacingXSmall
                    background: Rectangle {
                        color: Theme.colorChrome
                        radius: Theme.radiusSmall
                    }
                }
            }

            ThemedButton {
                contentPadding: Theme.spacingLarge
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                objectName: "meshMoreOptions"
                Layout.preferredWidth: Theme.controlHeight + Theme.spacingSmall
                text: "…"
                Accessible.name: qsTranslate("MeshTool", "More mesh options")
            }
        }

        GridLayout {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            Layout.rightMargin: Theme.spacingMedium
            columns: 2
            columnSpacing: Theme.spacingMedium
            rowSpacing: Theme.spacingSmall

            ThemedButton {
                contentPadding: Theme.spacingLarge
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                objectName: "meshAction"
                Layout.fillWidth: true
                text: qsTranslate("MeshTool", "Mesh")
                primaryAction: true
            }
            ThemedButton {
                contentPadding: Theme.spacingLarge
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                objectName: "meshHelpAction"
                Layout.fillWidth: true
                text: qsTranslate("MeshTool", "Help")
            }
            ThemedButton {
                contentPadding: Theme.spacingLarge
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                objectName: "meshPreviewAction"
                Layout.fillWidth: true
                text: qsTranslate("MeshTool", "Preview")
            }
            ThemedButton {
                contentPadding: Theme.spacingLarge
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                objectName: "meshCancelAction"
                Layout.fillWidth: true
                text: qsTranslate("DialogAction", "Cancel")
            }
        }

        ThemedCheckBox {

            wrapText: true
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            Layout.rightMargin: Theme.spacingMedium
            text: qsTranslate("MeshTool", "Remesh already meshed parts of the model")
        }
        ThemedCheckBox {

            wrapText: true
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            Layout.rightMargin: Theme.spacingMedium
            text: qsTranslate("MeshTool", "Place mesh in active layer")
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            Layout.rightMargin: Theme.spacingMedium
            Layout.bottomMargin: Theme.spacingMedium
            spacing: 0

            ThemedLabel {
                text: qsTranslate("MeshTool", "General")
                textSize: Theme.fontSmall
                leftPadding: Theme.spacingSmall
                rightPadding: Theme.spacingSmall
                topPadding: Theme.spacingXSmall
                bottomPadding: Theme.spacingXSmall
                background: Rectangle {
                    color: Theme.colorPanel
                    border.width: Theme.borderWidth
                    border.color: Theme.colorPanelLine
                    radius: Theme.radiusSmall
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: settings.implicitHeight + 2 * Theme.spacingLarge
                color: Theme.colorPanel
                border.width: Theme.borderWidth
                border.color: Theme.colorPanelLine

                ColumnLayout {
                    id: settings
                    anchors.fill: parent
                    anchors.margins: Theme.spacingLarge
                    spacing: Theme.spacingLarge

                    ThemedLabel {
                        Layout.fillWidth: true
                        text: qsTranslate("MeshTool", "Set curve edge lengths on the Curves tab. Set CAD edge lengths on the CAD tab or in Mesh > Density. The global edge length applies when geometry has no local edge length.")
                        textSize: Theme.fontSmall
                        textColor: Theme.colorTextMuted
                        wrapMode: Text.Wrap
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingSmall

                        ThemedLabel {
                            Layout.fillWidth: true
                            text: qsTranslate("MeshTool", "Global edge length:")
                            textSize: Theme.fontBody
                        }
                        ThemedTextField {
                            objectName: "globalEdgeLengthField"
                            Layout.preferredWidth: Theme.controlHeight * 3
                            Layout.preferredHeight: Theme.controlHeight
                            text: "12.00"
                            horizontalAlignment: TextInput.AlignRight
                            Accessible.name: qsTranslate("MeshTool", "Global edge length:")
                        }
                        ThemedLabel {
                            text: "mm"
                            textSize: Theme.fontBody
                        }
                    }

                    ThemedCheckBox {

                        wrapText: true
                        Layout.fillWidth: true
                        text: qsTranslate("MeshTool", "Match mesh")
                        checked: true
                    }
                    ThemedCheckBox {

                        wrapText: true
                        Layout.fillWidth: true
                        text: qsTranslate("MeshTool", "Calculate thickness for Dual Domain meshes")
                        checked: true
                    }
                }
            }
        }
    }
}

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
    ScrollBar.vertical.policy: ScrollBar.AsNeeded

    component MeshActionButton: Button {
        id: action

        property bool primaryAction: false

        hoverEnabled: true
        implicitHeight: Theme.controlHeight + Theme.spacingSmall
        font.pixelSize: Theme.fontBody

        background: Rectangle {
            color: action.primaryAction ? Theme.colorDialogPrimary : action.hovered ? Theme.colorHover : Theme.colorPanel
            border.width: action.visualFocus ? Theme.focusBorderWidth : Theme.borderWidth
            border.color: action.visualFocus ? Theme.colorFocus : action.primaryAction ? Theme.colorDialogPrimaryBorder : Theme.colorPanelLine
            radius: Theme.radiusSmall
        }

        contentItem: ThemedLabel {
            text: action.text
            textSize: Theme.fontBody
            font.weight: action.primaryAction ? Font.DemiBold : Font.Normal
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
    }

    component MeshOption: CheckBox {
        id: option

        hoverEnabled: true
        font.pixelSize: Theme.fontBody
        spacing: Theme.spacingSmall

        indicator: Rectangle {
            implicitWidth: Theme.iconSizeSmall
            implicitHeight: Theme.iconSizeSmall
            x: option.leftPadding
            y: (option.height - height) / 2
            color: option.checked ? Theme.colorFocus : Theme.colorPanel
            border.width: Theme.borderWidth
            border.color: option.checked ? Theme.colorFocus : Theme.colorPanelLine
            radius: Theme.radiusSmall

            ThemedLabel {
                anchors.centerIn: parent
                visible: option.checked
                text: "✓"
                textSize: Theme.fontSmall
                textColor: Theme.colorPanel
            }
        }

        contentItem: ThemedLabel {
            leftPadding: option.indicator.width + option.spacing
            text: option.text
            textSize: Theme.fontBody
            wrapMode: Text.Wrap
            verticalAlignment: Text.AlignVCenter
        }
    }

    ColumnLayout {
        width: meshTool.availableWidth
        spacing: Theme.spacingMedium

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            Layout.rightMargin: Theme.spacingMedium
            Layout.topMargin: Theme.spacingMedium
            spacing: Theme.spacingSmall

            ComboBox {
                id: operationSelect
                objectName: "meshOperationSelect"
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.controlHeight + Theme.spacingSmall
                model: [qsTranslate("MeshTool", "Generate Mesh")]
                Accessible.name: qsTranslate("MeshTool", "Mesh operation")
                leftPadding: Theme.spacingLarge
                rightPadding: Theme.spacingMedium
                topPadding: 0
                bottomPadding: 0

                background: Rectangle {
                    color: Theme.colorPanel
                    border.width: operationSelect.visualFocus ? Theme.focusBorderWidth : Theme.borderWidth
                    border.color: operationSelect.visualFocus ? Theme.colorFocus : Theme.colorPanelLine
                    radius: Theme.radiusSmall
                }
                indicator: Item {}
                contentItem: RowLayout {
                    spacing: Theme.spacingSmall

                    ThemedLabel {
                        Layout.fillWidth: true
                        text: operationSelect.displayText
                        textSize: Theme.fontBody
                        elide: Text.ElideRight
                    }
                    ThemedLabel {
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
                    ThemedIcon {
                        name: "caret"
                        iconSize: Theme.iconSizeCompact
                    }
                }
            }

            MeshActionButton {
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

            MeshActionButton {
                objectName: "meshAction"
                Layout.fillWidth: true
                text: qsTranslate("MeshTool", "Mesh")
                primaryAction: true
            }
            MeshActionButton {
                objectName: "meshHelpAction"
                Layout.fillWidth: true
                text: qsTranslate("MeshTool", "Help")
            }
            MeshActionButton {
                objectName: "meshPreviewAction"
                Layout.fillWidth: true
                text: qsTranslate("MeshTool", "Preview")
            }
            MeshActionButton {
                objectName: "meshCancelAction"
                Layout.fillWidth: true
                text: qsTranslate("DialogAction", "Cancel")
            }
        }

        MeshOption {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            Layout.rightMargin: Theme.spacingMedium
            text: qsTranslate("MeshTool", "Remesh already meshed parts of the model")
        }
        MeshOption {
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
                font.weight: Font.DemiBold
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

                    MeshOption {
                        Layout.fillWidth: true
                        text: qsTranslate("MeshTool", "Match mesh")
                        checked: true
                    }
                    MeshOption {
                        Layout.fillWidth: true
                        text: qsTranslate("MeshTool", "Calculate thickness for Dual Domain meshes")
                        checked: true
                    }
                }
            }
        }
    }
}

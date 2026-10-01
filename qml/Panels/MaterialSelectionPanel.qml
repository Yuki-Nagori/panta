// 内置材料选择与详情；材料摘要由 Rust 提供，面板只维护候选展示状态。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ScrollView {
    id: panel
    objectName: "materialSelectionPanel"
    property var material: ({})
    property bool detailsVisible: false
    property bool specificMode: false
    readonly property string materialText: material.sourceText ? qsTranslate("Material", material.sourceText) : ""
    clip: true
    contentWidth: availableWidth

    function reset() {
        detailsVisible = false;
        specificMode = false;
    }

    ColumnLayout {
        width: panel.availableWidth
        spacing: Theme.spacingMedium
        ThemedRadioButton {
            autoExclusive: false
            text: qsTranslate("MaterialDialog", "Commonly used materials")
            checked: !panel.specificMode
            onClicked: panel.specificMode = false
        }
        RowLayout {
            Layout.fillWidth: true
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.controlHeight * 2
                color: Theme.colorPanel
                border.color: Theme.colorPanelLine
                border.width: Theme.borderWidth
                radius: Theme.radiusSmall
                ThemedButton {
                    id: commonMaterial
                    contentPadding: Theme.spacingMedium
                    objectName: "defaultMaterialChoice"
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: Theme.borderWidth
                    text: panel.materialText
                    contentAlignLeft: true
                    enabled: !panel.specificMode
                    background: Rectangle {
                        color: Theme.colorSelected
                        border.width: commonMaterial.visualFocus ? Theme.focusBorderWidth : 0
                        border.color: Theme.colorFocus
                        radius: Theme.radiusSmall
                    }
                }
            }
            ThemedButton {
                contentPadding: Theme.spacingMedium
                text: qsTranslate("MaterialDialog", "Remove")
                enabled: false
            }
        }
        RowLayout {
            Layout.fillWidth: true
            ThemedRadioButton {
                autoExclusive: false
                text: qsTranslate("MaterialDialog", "Specific material")
                checked: panel.specificMode
                onClicked: panel.specificMode = true
            }
            Item {
                Layout.fillWidth: true
            }
            ThemedButton {
                contentPadding: Theme.spacingMedium
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                text: qsTranslate("MaterialDialog", "Customize Material List...")
                enabled: false
            }
            ThemedButton {
                contentPadding: Theme.spacingMedium
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                text: qsTranslate("MaterialDialog", "Reset Material List")
                enabled: false
            }
        }
        GridLayout {
            Layout.fillWidth: true
            columns: 3
            columnSpacing: Theme.spacingMedium
            rowSpacing: Theme.spacingSmall
            ThemedLabel {
                text: qsTranslate("MaterialDialog", "Polymer family")
            }
            ThemedComboBox {
                font.pixelSize: Theme.fontBody
                Layout.fillWidth: true
                model: [qsTranslate("Material", panel.material.familySourceText ?? "")]
                enabled: panel.specificMode
                Accessible.name: qsTranslate("MaterialDialog", "Polymer family")
            }
            ThemedButton {
                contentPadding: Theme.spacingMedium
                text: qsTranslate("MaterialDialog", "Import...")
                enabled: false
            }
            ThemedLabel {
                text: qsTranslate("MaterialDialog", "Material")
            }
            ThemedComboBox {
                font.pixelSize: Theme.fontBody
                Layout.fillWidth: true
                model: [panel.materialText]
                enabled: panel.specificMode
                Accessible.name: qsTranslate("MaterialDialog", "Material")
            }
            ThemedButton {
                contentPadding: Theme.spacingMedium
                text: qsTranslate("MaterialDialog", "Search...")
                enabled: false
            }
        }
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: selectedMaterial.implicitHeight + 2 * Theme.spacingMedium
            color: Theme.colorPanel
            border.color: Theme.colorPanelLine
            border.width: Theme.borderWidth
            radius: Theme.radiusSmall
            ColumnLayout {
                id: selectedMaterial
                anchors.fill: parent
                anchors.margins: Theme.spacingMedium
                spacing: Theme.spacingSmall
                ThemedLabel {
                    text: qsTranslate("MaterialDialog", "Selected material")
                    font.weight: Font.DemiBold
                }
                ThemedLabel {
                    Layout.fillWidth: true
                    text: panel.materialText
                    wrapMode: Text.Wrap
                }
                RowLayout {
                    ThemedButton {
                        contentPadding: Theme.spacingMedium
                        objectName: "materialDetailsAction"
                        text: qsTranslate("MaterialDialog", "Details...")
                        onClicked: panel.detailsVisible = !panel.detailsVisible
                    }
                    ThemedButton {
                        contentPadding: Theme.spacingMedium
                        text: qsTranslate("MaterialDialog", "Report...")
                        enabled: false
                    }
                }
                Repeater {
                    model: panel.detailsVisible ? (panel.material.properties ?? []) : []
                    delegate: RowLayout {
                        id: propertyRow
                        required property var modelData
                        Layout.fillWidth: true
                        ThemedLabel {
                            Layout.fillWidth: true
                            text: qsTranslate("MaterialProperty", propertyRow.modelData.sourceText)
                            textSize: Theme.fontSmall
                            wrapMode: Text.Wrap
                        }
                        ThemedLabel {
                            text: propertyRow.modelData.value
                            textSize: Theme.fontSmall
                        }
                    }
                }
                ThemedCheckBox {

                    font.pixelSize: Theme.fontSmall
                    text: qsTranslate("MaterialDialog", "Add material to commonly used list after selecting")
                    enabled: false
                }
            }
        }
    }
}

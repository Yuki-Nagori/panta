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

    component MaterialAction: ThemedToolButton {
        contentColor: Theme.colorText
        borderColor: Theme.colorPanelLine
        hoverColor: Theme.colorHover
        contentPadding: Theme.spacingMedium
    }

    component MaterialMode: RadioButton {
        id: mode
        // 模式由 specificMode 统一控制，避免同级控件的自动互斥再次修改 checked。
        autoExclusive: false
        implicitHeight: Theme.controlHeight
        padding: 0
        spacing: Theme.spacingSmall
        indicator: Rectangle {
            implicitWidth: Theme.iconSizeSmall
            implicitHeight: Theme.iconSizeSmall
            y: (mode.height - height) / 2
            radius: width / 2
            color: Theme.colorPanel
            border.width: mode.visualFocus ? Theme.focusBorderWidth : Theme.borderWidth
            border.color: mode.checked || mode.visualFocus ? Theme.colorFocus : Theme.colorPanelLine
            Rectangle {
                anchors.centerIn: parent
                width: Theme.iconSizeSmall / 2
                height: width
                radius: width / 2
                color: Theme.colorFocus
                visible: mode.checked
            }
        }
        contentItem: ThemedLabel {
            text: mode.text
            leftPadding: mode.indicator.width + mode.spacing
            verticalAlignment: Text.AlignVCenter
        }
    }

    component MaterialChoice: ComboBox {
        id: choice
        implicitHeight: Theme.controlHeight
        leftPadding: Theme.spacingMedium
        rightPadding: Theme.iconSizeSmall + 2 * Theme.spacingSmall
        opacity: enabled ? 1 : Theme.disabledOpacity
        contentItem: ThemedLabel {
            text: choice.displayText
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        indicator: ThemedIcon {
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            name: "caret"
            iconSize: Theme.iconSizeSmall
            color: Theme.colorIcon
        }
        background: Rectangle {
            color: Theme.colorPanel
            border.color: choice.visualFocus ? Theme.colorFocus : Theme.colorPanelLine
            border.width: choice.visualFocus ? Theme.focusBorderWidth : Theme.borderWidth
            radius: Theme.radiusSmall
        }
    }

    component MaterialFavorite: CheckBox {
        id: favorite
        implicitHeight: Theme.controlHeight
        padding: 0
        spacing: Theme.spacingSmall
        opacity: enabled ? 1 : Theme.disabledOpacity
        indicator: Rectangle {
            implicitWidth: Theme.iconSizeSmall
            implicitHeight: Theme.iconSizeSmall
            y: (favorite.height - height) / 2
            color: favorite.checked ? Theme.colorFocus : Theme.colorPanel
            border.color: favorite.checked ? Theme.colorFocus : Theme.colorPanelLine
            border.width: Theme.borderWidth
            radius: Theme.radiusSmall
            ThemedLabel {
                anchors.centerIn: parent
                text: "✓"
                visible: favorite.checked
                textColor: Theme.colorPanel
                textSize: Theme.fontSmall
            }
        }
        contentItem: ThemedLabel {
            text: favorite.text
            leftPadding: favorite.indicator.width + favorite.spacing
            textSize: Theme.fontSmall
            verticalAlignment: Text.AlignVCenter
        }
    }

    ColumnLayout {
        width: panel.availableWidth
        spacing: Theme.spacingMedium
        MaterialMode {
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
                MaterialAction {
                    id: commonMaterial
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
            MaterialAction {
                text: qsTranslate("MaterialDialog", "Remove")
                enabled: false
            }
        }
        RowLayout {
            Layout.fillWidth: true
            MaterialMode {
                text: qsTranslate("MaterialDialog", "Specific material")
                checked: panel.specificMode
                onClicked: panel.specificMode = true
            }
            Item {
                Layout.fillWidth: true
            }
            MaterialAction {
                text: qsTranslate("MaterialDialog", "Customize Material List...")
                enabled: false
            }
            MaterialAction {
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
            MaterialChoice {
                Layout.fillWidth: true
                model: [qsTranslate("Material", panel.material.familySourceText ?? "")]
                enabled: panel.specificMode
                Accessible.name: qsTranslate("MaterialDialog", "Polymer family")
            }
            MaterialAction {
                text: qsTranslate("MaterialDialog", "Import...")
                enabled: false
            }
            ThemedLabel {
                text: qsTranslate("MaterialDialog", "Material")
            }
            MaterialChoice {
                Layout.fillWidth: true
                model: [panel.materialText]
                enabled: panel.specificMode
                Accessible.name: qsTranslate("MaterialDialog", "Material")
            }
            MaterialAction {
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
                    MaterialAction {
                        objectName: "materialDetailsAction"
                        text: qsTranslate("MaterialDialog", "Details...")
                        onClicked: panel.detailsVisible = !panel.detailsVisible
                    }
                    MaterialAction {
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
                MaterialFavorite {
                    text: qsTranslate("MaterialDialog", "Add material to commonly used list after selecting")
                    enabled: false
                }
            }
        }
    }
}

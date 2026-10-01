// 左栏 Layers Dock；只呈现工具栏和页签，图层内容与动作待后续定义。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel

    property bool dockOpen: true
    property var importedPartNames: []
    readonly property var layerTools: [
        {
            icon: "new",
            preserveIconColors: true,
            label: qsTranslate("LayerAction", "New layer"),
            action: "new"
        },
        {
            icon: "open",
            preserveIconColors: true,
            label: qsTranslate("LayerAction", "Open layer"),
            action: "open"
        },
        {
            icon: "check",
            label: qsTranslate("LayerAction", "Validate layer"),
            action: "validate"
        },
        {
            icon: "wizard",
            label: qsTranslate("LayerAction", "Edit layer"),
            action: "edit"
        },
        {
            icon: "delete",
            label: qsTranslate("LayerAction", "Remove layer"),
            action: "remove"
        },
        {
            icon: "copy",
            label: qsTranslate("LayerAction", "Layer options"),
            action: "options"
        },
        {
            icon: "export",
            label: qsTranslate("LayerAction", "Move layer"),
            action: "move"
        },
        {
            icon: "close",
            label: qsTranslate("LayerAction", "Close layer tools"),
            action: "close-tools"
        }
    ]

    signal closeRequested
    signal toolRequested(string action)

    implicitWidth: Theme.leftPanelMinimumWidth
    visible: dockOpen

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            objectName: "layersToolbar"
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.panelToolbarHeight
            color: Theme.colorChrome

            Row {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                anchors.leftMargin: Theme.spacingXSmall
                anchors.rightMargin: Theme.paneCloseSize + Theme.spacingMedium
                spacing: Theme.spacingTiny

                Repeater {
                    model: panel.layerTools

                    delegate: ThemedToolButton {
                        required property var modelData

                        width: Theme.toolbarButtonWidth
                        controlHeight: Theme.toolbarButtonHeight
                        contentPadding: 0
                        iconSize: Theme.iconSizeSmall
                        iconName: modelData.icon
                        preserveIconColors: modelData.preserveIconColors === true
                        accessibleName: modelData.label
                        onClicked: panel.toolRequested(modelData.action)
                    }
                }
            }
        }

        Rectangle {
            objectName: "layersTabDivider"
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.borderWidth
            color: Theme.colorPanelLine
        }

        Rectangle {
            objectName: "layersTabRow"
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.panelToolbarHeight - 2 * Theme.spacingTiny
            color: Theme.colorChrome
            visible: panel.importedPartNames.length > 0

            ThemedToolButton {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: Theme.layersTabWidth
                text: qsTranslate("LayerPanelTitle", "Layers")
                iconName: "layers"
                highlighted: true
                contentAlignLeft: true
                contentPadding: Theme.spacingSmall
                controlHeight: parent.height
                iconSize: Theme.iconSizeSmall
                contentColor: Theme.colorText
                accessibleName: qsTranslate("LayerPanelTitle", "Layers")
            }
        }

        Item {
            objectName: "layersContent"
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

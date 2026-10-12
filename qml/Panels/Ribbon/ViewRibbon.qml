// View Ribbon 展示视图与窗口工具；未接服务的控件保持禁用。
pragma ComponentBehavior: Bound

import QtQuick

Row {
    id: viewRibbon

    component CompactAction: ThemedToolButton {
        controlHeight: Theme.ribbonCompactHeight
        contentPadding: Theme.spacingTiny
        iconSize: Theme.ribbonCompactIconSize
        preserveIconColors: true
        contentColor: Theme.colorText
        font.pixelSize: Theme.fontRibbon
        enabled: false
    }

    component BrowseAction: CompactAction {
        width: Theme.viewBrowseActionWidth
        contentAlignLeft: true
    }

    RibbonGroup {
        title: qsTranslate("ViewRibbon", "Appearance")
        width: Theme.viewAppearanceGroupWidth
        Row {
            spacing: Theme.spacingTiny

            RibbonTile {
                objectName: "ribbon-view-solid"
                width: Theme.viewWindowTileWidth
                iconName: "view-solid"
                text: qsTranslate("ViewRibbon", "Solid")
                enabled: false
            }
            Rectangle {
                objectName: "ribbon-view-model-display"
                width: Theme.viewWindowTileWidth
                height: Theme.ribbonToolHeight
                color: Theme.colorHover
                radius: Theme.radiusSmall
                Column {
                    objectName: "ribbonTileContent"
                    anchors.centerIn: parent
                    spacing: Theme.ribbonContentSpacing
                    ThemedIcon {
                        anchors.horizontalCenter: parent.horizontalCenter
                        name: "view-model-display"
                        iconSize: Theme.iconSizeRibbon
                        preserveSourceColors: true
                    }
                    ThemedLabel {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: parent.parent.width - 2 * Theme.spacingTiny
                        height: Theme.ribbonTextLines * Theme.ribbonTextLineHeight
                        text: qsTranslate("ViewRibbon", "Model\nDisplay")
                        textSize: Theme.fontRibbon
                        textColor: Theme.colorText
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignTop
                        lineHeightMode: Text.FixedHeight
                        lineHeight: Theme.ribbonTextLineHeight
                    }
                }
                Accessible.name: qsTranslate("ViewRibbon", "Model Display")
                Accessible.role: Accessible.RadioButton
                Accessible.checked: true
                Accessible.description: qsTranslate("ViewRibbon", "Current display mode")
            }
            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.borderWidth
                CompactAction {
                    width: Theme.viewAppearanceControlWidth
                    iconName: "view-perspective"
                    text: qsTranslate("ViewRibbon", "Perspective")
                    accessibleName: text
                }
                RibbonComboBox {
                    objectName: "ribbon-units-combo"
                    width: Theme.viewUnitsComboWidth
                    anchors.horizontalCenter: parent.horizontalCenter
                    model: [qsTranslate("ViewRibbon", "Metric Units")]
                    currentIndex: 0
                    enabled: false
                    Accessible.name: qsTranslate("ViewRibbon", "Units")
                }
            }
        }
    }

    RibbonGroup {
        title: qsTranslate("ViewRibbon", "Window")
        width: Theme.viewWindowGroupWidth
        Row {
            spacing: Theme.spacingTiny
            RibbonTile {
                width: Theme.viewWindowTileWidth
                iconName: "view-interface"
                text: qsTranslate("ViewRibbon", "User\nInterface")
                enabled: false
            }
            RibbonTile {
                width: Theme.viewWindowTileWidth
                iconName: "view-clean-screen"
                text: qsTranslate("ViewRibbon", "Clean\nScreen")
                enabled: false
            }
            RibbonTile {
                width: Theme.viewWindowTileWidth
                iconName: "view-switch"
                text: qsTranslate("ViewRibbon", "Switch")
                showCaret: true
                enabled: false
            }
            Grid {
                anchors.verticalCenter: parent.verticalCenter
                columns: 2
                spacing: Theme.borderWidth
                Repeater {
                    model: [
                        {
                            key: "tile",
                            icon: "view-tile",
                            label: qsTranslate("ViewRibbon", "Tile"),
                            caret: true
                        },
                        {
                            key: "new",
                            icon: "view-new-window",
                            label: qsTranslate("ViewRibbon", "New"),
                            caret: false
                        },
                        {
                            key: "split",
                            icon: "view-split",
                            label: qsTranslate("ViewRibbon", "Split"),
                            caret: true
                        },
                        {
                            key: "close",
                            icon: "view-close-window",
                            label: qsTranslate("ViewRibbon", "Close"),
                            caret: true
                        },
                        {
                            key: "cascade",
                            icon: "view-cascade",
                            label: qsTranslate("ViewRibbon", "Cascade"),
                            caret: false
                        },
                        {
                            key: "arrange",
                            icon: "view-arrange",
                            label: qsTranslate("ViewRibbon", "Arrange"),
                            caret: false
                        }
                    ]
                    delegate: CompactAction {
                        required property var modelData
                        objectName: "ribbon-view-window-" + modelData.key
                        width: Theme.viewCompactActionWidth
                        iconName: modelData.icon
                        text: modelData.label
                        showCaret: modelData.caret
                        accessibleName: modelData.label
                        contentAlignLeft: true
                    }
                }
            }
        }
    }

    RibbonGroup {
        title: qsTranslate("ViewRibbon", "Locking")
        width: Theme.viewLockingGroupWidth
        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.borderWidth
            CompactAction {
                width: Theme.viewLockActionWidth
                iconName: "view-lock"
                text: qsTranslate("ViewRibbon", "Lock View")
                accessibleName: text
                showCaret: true
            }
            CompactAction {
                width: Theme.viewLockActionWidth
                iconName: "view-lock-point"
                text: qsTranslate("ViewRibbon", "Lock Point")
                accessibleName: text
                showCaret: true
            }
            CompactAction {
                width: Theme.viewLockActionWidth
                iconName: "view-lock-animation"
                text: qsTranslate("ViewRibbon", "Lock Animation")
                accessibleName: text
                showCaret: true
            }
        }
    }

    RibbonGroup {
        title: qsTranslate("ViewRibbon", "Browse")
        width: Theme.viewBrowseGroupWidth
        Row {
            spacing: Theme.spacingTiny
            RibbonTile {
                iconName: "view-navigation"
                text: qsTranslate("ViewRibbon", "Global\nNavigation\nControl")
                enabled: false
            }
            Grid {
                anchors.verticalCenter: parent.verticalCenter
                columns: 3
                spacing: Theme.borderWidth
                Repeater {
                    model: [
                        {
                            icon: "view-select",
                            label: qsTranslate("ViewRibbon", "Select")
                        },
                        {
                            icon: "view-orbit",
                            label: qsTranslate("ViewRibbon", "Dynamic Orbit")
                        },
                        {
                            icon: "view-previous",
                            label: qsTranslate("ViewRibbon", "Previous View")
                        },
                        {
                            icon: "view-pan",
                            label: qsTranslate("ViewRibbon", "Pan")
                        },
                        {
                            icon: "view-center",
                            label: qsTranslate("ViewRibbon", "Center")
                        },
                        {
                            icon: "view-front",
                            label: qsTranslate("ViewRibbon", "Front View")
                        },
                        {
                            icon: "view-zoom",
                            label: qsTranslate("ViewRibbon", "Zoom All")
                        },
                        {
                            icon: "view-measure",
                            label: qsTranslate("ViewRibbon", "Measure")
                        },
                        {
                            icon: "view-cube",
                            label: qsTranslate("ViewRibbon", "View Cube")
                        }
                    ]
                    delegate: BrowseAction {
                        required property var modelData
                        iconName: modelData.icon
                        text: modelData.label
                        accessibleName: modelData.label
                    }
                }
            }
        }
    }

    RibbonGroup {
        title: qsTranslate("ViewRibbon", "Cutting Plane")
        width: Theme.viewCuttingPlaneGroupWidth
        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.borderWidth
            CompactAction {
                iconName: "view-edit"
                text: qsTranslate("ViewRibbon", "Edit")
                accessibleName: qsTranslate("ViewRibbon", "Edit Cutting Plane")
            }
            CompactAction {
                iconName: "view-move"
                text: qsTranslate("ViewRibbon", "Move")
                accessibleName: qsTranslate("ViewRibbon", "Move Cutting Plane")
            }
        }
    }

    RibbonGroup {
        title: qsTranslate("ViewRibbon", "View")
        width: Theme.viewControlsGroupWidth
        Row {
            spacing: Theme.spacingTiny
            RibbonTile {
                iconName: "view-edit"
                text: qsTranslate("ViewRibbon", "Edit")
                Accessible.name: qsTranslate("ViewRibbon", "Edit View")
                enabled: false
            }
            Column {
                id: viewControlsStack
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.borderWidth
                readonly property int labelColumnWidth: Math.ceil(Math.max(rotationAngleLabel.implicitWidth, viewNameLabel.implicitWidth))
                Row {
                    spacing: Theme.spacingTiny
                    ThemedLabel {
                        id: rotationAngleLabel
                        anchors.verticalCenter: parent.verticalCenter
                        width: viewControlsStack.labelColumnWidth
                        text: qsTranslate("ViewRibbon", "Rotation Angle")
                        textSize: Theme.fontRibbon
                    }
                    ThemedTextField {
                        width: Theme.viewControlFieldWidth
                        height: Theme.ribbonComboBoxHeight
                        text: "0 0 0"
                        readOnly: true
                        enabled: false
                        Accessible.name: qsTranslate("ViewRibbon", "Rotation Angle")
                    }
                }
                Row {
                    spacing: Theme.spacingTiny
                    ThemedLabel {
                        id: viewNameLabel
                        anchors.verticalCenter: parent.verticalCenter
                        width: viewControlsStack.labelColumnWidth
                        text: qsTranslate("ViewRibbon", "View Name")
                        textSize: Theme.fontRibbon
                    }
                    RibbonComboBox {
                        objectName: "ribbon-view-name-combo"
                        width: Theme.viewControlFieldWidth
                        model: []
                        enabled: false
                        Accessible.name: qsTranslate("ViewRibbon", "View Name")
                    }
                }
                Row {
                    spacing: Theme.spacingXSmall
                    CompactAction {
                        iconName: "view-save"
                        text: qsTranslate("ViewRibbon", "Save")
                        accessibleName: qsTranslate("ViewRibbon", "Save View")
                    }
                    CompactAction {
                        iconName: "view-delete"
                        text: qsTranslate("ViewRibbon", "Delete")
                        accessibleName: qsTranslate("ViewRibbon", "Delete View")
                    }
                }
            }
        }
    }
}

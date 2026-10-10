// 已接入的充填动画控制由 AnalysisModel 驱动；其余结果工具给出统一提示。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls

Row {
    id: resultsRibbon
    property var analysisModel: null
    readonly property bool animationAvailable: analysisModel && analysisModel.animationAvailable
    component CompactAction: ThemedToolButton {
        controlHeight: Theme.ribbonCompactHeight
        contentPadding: Theme.spacingTiny
        iconSize: Theme.ribbonCompactIconSize
        preserveIconColors: true
        contentColor: Theme.colorText
        font.pixelSize: Theme.fontRibbon
    }

    component ActionStack: Column {
        id: actionStack
        required property var actions
        spacing: Theme.borderWidth
        Repeater {
            model: actionStack.actions
            delegate: CompactAction {
                required property var modelData
                text: modelData.label
                iconName: modelData.icon
                contentAlignLeft: true
            }
        }
    }

    component ActionGrid: Grid {
        id: actionGrid
        required property var actions
        spacing: Theme.borderWidth
        Repeater {
            model: actionGrid.actions
            delegate: CompactAction {
                required property var modelData
                width: Theme.ribbonCompactWidth
                iconName: modelData.icon
                accessibleName: modelData.label
                enabled: modelData.key === undefined || resultsRibbon.animationAvailable
                clickAction: () => {
                    switch (modelData.key) {
                    case "play":
                        resultsRibbon.analysisModel.play();
                        break;
                    case "pause":
                        resultsRibbon.analysisModel.pause();
                        break;
                    case "stop":
                        resultsRibbon.analysisModel.stop();
                        break;
                    case "first":
                        resultsRibbon.analysisModel.playbackTime = 0;
                        break;
                    case "last":
                        resultsRibbon.analysisModel.playbackTime = resultsRibbon.analysisModel.duration;
                        break;
                    default:
                        FeatureNotice.notify(Window.window);
                        break;
                    }
                }
            }
        }
    }

    component ResultSlider: Slider {
        id: slider
        enabled: false
        height: Theme.ribbonCompactHeight
        leftPadding: Theme.spacingTiny
        rightPadding: Theme.spacingTiny
        opacity: Theme.disabledOpacity
        background: Rectangle {
            x: slider.leftPadding
            y: (slider.height - height) / 2
            width: slider.availableWidth
            height: Theme.spacingXSmall
            radius: Theme.radiusSmall
            color: Theme.colorPanelLine
        }
        handle: Rectangle {
            x: slider.leftPadding + slider.visualPosition * (slider.availableWidth - width)
            y: (slider.height - height) / 2
            width: Theme.iconSizeCompact
            height: width
            radius: width / 2
            color: Theme.colorTextMuted
        }
    }

    RibbonGroup {
        title: qsTr("Plots")
        RibbonTile {
            iconName: "results-new-plot"
            text: qsTr("New Plot")

            showCaret: true
        }
        ActionStack {
            actions: [
                {
                    icon: "results-notes",
                    label: qsTr("Notes")
                },
                {
                    icon: "results-xy-curve",
                    label: qsTr("Add XY Curve")
                }
            ]
        }
    }
    RibbonGroup {
        title: qsTr("Properties")
        RibbonTile {
            iconName: "results-plot-properties"
            text: qsTr("Plot\nProperties")
        }
        RibbonTile {
            iconName: "results-save-defaults"
            text: qsTr("Save\nDefaults")

            showCaret: true
        }
    }
    RibbonGroup {
        title: qsTr("Animation")
        Column {
            width: Theme.ribbonResultsControlWidth
            ActionGrid {
                anchors.horizontalCenter: parent.horizontalCenter
                columns: 7
                actions: [
                    {
                        icon: "results-first",
                        key: "first",
                        label: qsTr("First frame")
                    },
                    {
                        icon: "results-last",
                        key: "last",
                        label: qsTr("Last frame")
                    },
                    {
                        icon: "results-play",
                        key: "play",
                        label: qsTr("Play")
                    },
                    {
                        icon: "results-pause",
                        key: "pause",
                        label: qsTr("Pause")
                    },
                    {
                        icon: "results-stop",
                        key: "stop",
                        label: qsTr("Stop")
                    },
                    {
                        icon: "results-loop",
                        label: qsTr("Loop")
                    },
                    {
                        icon: "results-ping-pong",
                        label: qsTr("Ping-pong")
                    }
                ]
            }
            Row {
                spacing: Theme.spacingXSmall
                ResultSlider {
                    width: Theme.ribbonResultsControlWidth - timeLabel.width - parent.spacing
                    enabled: resultsRibbon.animationAvailable
                    opacity: enabled ? 1 : Theme.disabledOpacity
                    from: 0
                    to: resultsRibbon.analysisModel ? resultsRibbon.analysisModel.duration : 1
                    value: resultsRibbon.analysisModel ? resultsRibbon.analysisModel.playbackTime : 0
                    onMoved: resultsRibbon.analysisModel.playbackTime = value
                    Accessible.name: qsTr("Animation time")
                }
                ThemedLabel {
                    id: timeLabel
                    width: Theme.ribbonTimeDisplayWidth
                    text: resultsRibbon.analysisModel && resultsRibbon.analysisModel.resultReady ? resultsRibbon.analysisModel.playbackTime.toFixed(3) + " s" : ""
                    textSize: Theme.fontRibbon
                    textColor: Theme.colorTextMuted
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
    }
    RibbonGroup {
        title: qsTr("Examine")
        RibbonTile {
            iconName: "results-examine"
            text: qsTr("Examine")
        }
        RibbonTile {
            iconName: "results-min-max"
            text: qsTr("Show\nMin/Max")
        }
    }
    RibbonGroup {
        title: qsTr("Histogram")
        RibbonTile {
            iconName: "results-histogram"
            text: qsTr("Histogram")
        }
    }
    RibbonGroup {
        title: qsTr("Scaling")
        Column {
            width: Theme.ribbonResultsControlWidth
            Row {
                spacing: Theme.spacingXSmall
                ThemedLabel {
                    id: scaleMinimum
                    width: Theme.ribbonScaleValueWidth
                    text: ""
                    textSize: Theme.fontRibbon
                    anchors.verticalCenter: parent.verticalCenter
                }
                ThemedIcon {
                    name: "results-scale-range"
                    preserveSourceColors: true
                    width: Theme.ribbonResultsControlWidth - scaleMinimum.width - scaleMaximum.width - 2 * parent.spacing
                    height: Theme.ribbonCompactHeight
                    sourceSize: Qt.size(width, height)
                    opacity: Theme.disabledOpacity
                }
                ThemedLabel {
                    id: scaleMaximum
                    width: Theme.ribbonScaleValueWidth
                    text: ""
                    textSize: Theme.fontRibbon
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
            Row {
                spacing: Theme.spacingXSmall
                CompactAction {
                    width: (Theme.ribbonResultsControlWidth - parent.spacing) / 2
                    iconName: "results-scale"
                    text: qsTr("Set Scale")
                }
                CompactAction {
                    width: (Theme.ribbonResultsControlWidth - parent.spacing) / 2
                    iconName: "results-reset-scale"
                    text: qsTr("Reset Scale")
                }
            }
        }
    }
    RibbonGroup {
        title: qsTr("Warpage")
        ActionStack {
            actions: [
                {
                    icon: "results-warpage",
                    label: qsTr("Visualize")
                },
                {
                    icon: "results-warpage",
                    label: qsTr("Restore")
                }
            ]
        }
    }
    RibbonGroup {
        title: qsTr("Export and Publish")
        RibbonTile {
            iconName: "results-defect"
            text: qsTr("Defect\nVisualization")
        }
        RibbonTile {
            iconName: "results-export"
            text: qsTr("Export\nResults")
        }
        ActionStack {
            actions: [
                {
                    icon: "results-mark",
                    label: qsTr("Mark")
                },
                {
                    icon: "results-unmark",
                    label: qsTr("Unmark")
                },
                {
                    icon: "results-xy-curve",
                    label: qsTr("XY Plot")
                }
            ]
        }
    }
    RibbonGroup {
        title: qsTr("Cutting Plane")
        ActionStack {
            actions: [
                {
                    icon: "results-cutting-plane",
                    label: qsTr("Edit")
                },
                {
                    icon: "results-cutting-plane",
                    label: qsTr("Move")
                }
            ]
        }
    }
    RibbonGroup {
        title: qsTr("Windows")
        ActionGrid {
            columns: 2
            actions: [
                {
                    icon: "results-tile-horizontal",
                    label: qsTr("Tile horizontally")
                },
                {
                    icon: "results-window-grid",
                    label: qsTr("Arrange windows")
                },
                {
                    icon: "results-tile-vertical",
                    label: qsTr("Tile vertically")
                },
                {
                    icon: "results-window-horizontal",
                    label: qsTr("Split horizontally")
                },
                {
                    icon: "results-window-sync",
                    label: qsTr("Synchronize windows")
                },
                {
                    icon: "results-window-vertical",
                    label: qsTr("Split vertically")
                }
            ]
        }
    }
    RibbonGroup {
        title: qsTr("Locking")
        ActionGrid {
            columns: 3
            actions: [
                {
                    icon: "results-lock-window",
                    label: qsTr("Lock window")
                },
                {
                    icon: "results-lock-legend",
                    label: qsTr("Lock legend")
                },
                {
                    icon: "results-lock-mesh",
                    label: qsTr("Lock mesh")
                },
                {
                    icon: "results-sync-lock-window",
                    label: qsTr("Synchronize lock window")
                },
                {
                    icon: "results-sync-lock-legend",
                    label: qsTr("Synchronize lock legend")
                },
                {
                    icon: "results-sync-lock-mesh",
                    label: qsTr("Synchronize lock mesh")
                },
                {
                    icon: "results-release-lock-window",
                    label: qsTr("Release lock window")
                },
                {
                    icon: "results-release-lock-legend",
                    label: qsTr("Release lock legend")
                },
                {
                    icon: "results-release-lock-mesh",
                    label: qsTr("Release lock mesh")
                }
            ]
        }
    }
    Item {
        width: Theme.ribbonTrailingWidth
        height: Theme.ribbonHeight - Theme.borderWidth
    }
}

// 两种工程状态共用 Ribbon 结构；仅新建 / 打开接现有命令，其余入口为视觉参考。
pragma ComponentBehavior: Bound

import QtQuick

Rectangle {
    id: panel
    objectName: "ribbonPanel"

    property bool projectOpen: false
    signal newProjectRequested
    signal openProjectRequested

    readonly property var groups: projectOpen ? [
        {
            title: qsTranslate("RibbonGroupImport", "Import"),
            caret: false,
            tools: [
                {
                    key: "import",
                    label: qsTranslate("RibbonActionImport", "Import"),
                    icon: "project-import"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupCreate", "Create"),
            caret: false,
            tools: [
                {
                    key: "add",
                    label: qsTranslate("RibbonActionAdd", "Add"),
                    icon: "document-new"
                },
                {
                    key: "dual-domain",
                    label: qsTranslate("RibbonActionDualDomain", "Dual\nDomain"),
                    icon: "domain-dual",
                    caret: true
                },
                {
                    key: "geometry",
                    label: qsTranslate("RibbonActionGeometry", "Geometry"),
                    icon: "geometry"
                },
                {
                    key: "mesh",
                    label: qsTranslate("RibbonActionMesh", "Mesh"),
                    icon: "mesh"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupSetup", "Molding Process Setup"),
            caret: false,
            tools: [
                {
                    key: "molding",
                    label: qsTranslate("RibbonActionMolding", "Thermoplastics\nInjection Molding"),
                    icon: "molding"
                },
                {
                    key: "sequence",
                    label: qsTranslate("RibbonActionSequence", "Analysis\nSequence"),
                    icon: "analysis-sequence"
                },
                {
                    key: "material",
                    label: qsTranslate("RibbonActionMaterial", "Select\nMaterial"),
                    icon: "material"
                },
                {
                    key: "injection",
                    label: qsTranslate("RibbonActionInjection", "Injection\nLocations"),
                    icon: "injection-location"
                },
                {
                    key: "settings",
                    label: qsTranslate("RibbonActionSettings", "Process\nSettings"),
                    icon: "process-settings"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupAnalysis", "Results"),
            caret: false,
            tools: [
                {
                    key: "optimization",
                    label: qsTranslate("RibbonActionOptimization", "Optimization"),
                    icon: "optimization"
                },
                {
                    key: "boundary",
                    label: qsTranslate("RibbonActionBoundary", "Boundary\nConditions"),
                    icon: "boundary-conditions"
                },
                {
                    key: "analyze",
                    label: qsTranslate("RibbonActionAnalyze", "Analyze"),
                    icon: "analysis-run",
                    enabled: false
                },
                {
                    key: "logs",
                    label: qsTranslate("RibbonActionLogs", "Logs"),
                    icon: "document-report"
                },
                {
                    key: "jobs",
                    label: qsTranslate("RibbonActionJobs", "Job\nManager"),
                    icon: "job-manager"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupReporting", "Reporting"),
            caret: false,
            tools: [
                {
                    key: "results",
                    label: qsTranslate("RibbonActionResults", "Results"),
                    icon: "analysis-results"
                },
                {
                    key: "reports",
                    label: qsTranslate("RibbonActionReports", "Reports"),
                    icon: "document-report"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupShare", "Share"),
            caret: false,
            tools: [
                {
                    key: "shared-views",
                    label: qsTranslate("RibbonActionSharedViews", "Shared\nViews"),
                    icon: "shared-views"
                }
            ]
        }
    ] : [
        {
            title: qsTranslate("RibbonGroupStart", "Start"),
            caret: true,
            tools: [
                {
                    key: "new-project",
                    label: qsTranslate("RibbonActionNewProject", "New\nProject"),
                    icon: "document-new"
                },
                {
                    key: "open-project",
                    label: qsTranslate("RibbonActionOpenProject", "Open\nProject"),
                    icon: "document-open"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupFeatures", "New Features"),
            caret: false,
            tools: [
                {
                    key: "new-features",
                    label: qsTranslate("RibbonActionNewFeatures", "New\nFeatures"),
                    icon: "ribbon-whatsnew"
                }
            ]
        },
        {
            title: qsTranslate("RibbonGroupLearn", "Learn"),
            caret: false,
            tools: [
                {
                    key: "start-here",
                    label: qsTranslate("RibbonActionStartHere", "Start Here"),
                    icon: "ribbon-start"
                },
                {
                    key: "tutorials",
                    label: qsTranslate("RibbonActionTutorials", "Tutorials"),
                    icon: "ribbon-learn"
                },
                {
                    key: "videos",
                    label: qsTranslate("RibbonActionVideos", "Videos"),
                    icon: "media-video"
                },
                {
                    key: "help",
                    label: qsTranslate("RibbonActionHelp", "Help"),
                    icon: "help-browser"
                }
            ]
        }
    ]

    implicitHeight: Theme.ribbonHeight
    gradient: Gradient {
        GradientStop {
            position: 0
            color: Theme.colorRibbonTop
        }
        GradientStop {
            position: 1
            color: Theme.colorRibbonBottom
        }
    }

    HorizontalToolStrip {
        id: ribbonStrip
        objectName: "ribbonStrip"
        anchors.fill: parent
        anchors.bottomMargin: Theme.borderWidth
        contentRoot: ribbonContent

        Row {
            id: ribbonContent
            height: ribbonStrip.height

            Repeater {
                model: panel.groups
                delegate: RibbonGroup {
                    id: group
                    required property var modelData
                    title: modelData.title
                    showCaret: modelData.caret

                    Repeater {
                        model: group.modelData.tools
                        delegate: RibbonTile {
                            required property var modelData
                            objectName: "ribbon-" + modelData.key
                            text: modelData.label
                            iconName: modelData.icon
                            showCaret: modelData.caret === true
                            enabled: modelData.enabled !== false
                            onClicked: {
                                if (modelData.key === "new-project")
                                    panel.newProjectRequested();
                                else if (modelData.key === "open-project")
                                    panel.openProjectRequested();
                            }
                        }
                    }
                }
            }
            Item {
                width: Theme.ribbonTrailingWidth
                height: ribbonStrip.height
            }
        }
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: Theme.borderWidth
        color: Theme.colorChromeLine
    }
}

// 工程目录与当前方案任务；数据由侧栏宿主显式传入。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: panel
    objectName: "projectTasksPage"
    property bool projectOpen: false
    property string projectName: ""
    property var importedPartNames: []
    // 稳定导入记录 ID，与 importedPartNames 按下标对应；行点击即请求
    // 打开 / 激活对应视口文档（080）。
    property var importedPartIds: []
    property string activeDocumentId: ""
    property string activeDocumentTitle: ""
    property bool importedPartAvailable: importedPartNames.length > 0
    property string importedPartName: ""
    property string analysisSequenceText: ""
    property string analysisSequenceId: ""
    property string materialText: ""
    property string materialId: ""
    property bool fillSettingsConfirmed: false
    property bool gateLocationSettingsConfirmed: false
    readonly property bool processSettingsConfirmed: analysisSequenceId === "fill" ? fillSettingsConfirmed : analysisSequenceId === "gate-location" && gateLocationSettingsConfirmed
    readonly property bool processSettingsAvailable: analysisSequenceId === "fill" || analysisSequenceId === "gate-location"
    property string meshType: ""
    property var meshTypes: []
    property bool logsOpen: false
    property int logRunCount: 0
    property var resultRun: null

    function meshTypeText() {
        const entry = meshTypes.find(entry => entry.id === meshType);
        return entry ? qsTranslate("MeshType", entry.sourceText) : meshType;
    }
    readonly property var planTaskItems: [
        {
            id: "imported-part",
            text: qsTranslate("ImportTask", "Part (%1)").arg(panel.activePartTitle),
            icon: "stl-file",
            completed: true
        },
        {
            id: "create-mesh",
            text: panel.meshType.length > 0 ? qsTranslate("ImportTask", "Mesh (%1)").arg(panel.meshTypeText()) : qsTranslate("ImportTask", "Create Mesh..."),
            icon: "task-mesh",
            completed: false
        },
        {
            id: "analysis-sequence",
            text: panel.analysisSequenceText,
            icon: "task-analysis-sequence",
            completed: panel.analysisSequenceId.length > 0
        },
        {
            id: "material-data",
            text: panel.materialText.length > 0 ? panel.materialText : qsTranslate("ImportTask", "Material Data"),
            icon: "task-material",
            completed: panel.materialId.length > 0
        },
        {
            id: "injection-locations",
            text: qsTranslate("ImportTask", "Set Injection Locations..."),
            icon: "task-injection",
            completed: false
        },
        {
            id: "process-settings",
            text: panel.processSettingsConfirmed ? qsTranslate("ProcessTask", "Process Settings (Custom)") : qsTranslate("ProcessTask", "Process Settings (Default)"),
            icon: "task-settings",
            completed: panel.processSettingsConfirmed,
            enabled: panel.processSettingsAvailable
        },
        {
            id: "optimization",
            text: qsTranslate("ImportTask", "Optimization (None)"),
            icon: "task-optimization",
            completed: false
        },
        {
            id: "analyze",
            text: qsTranslate("UiCommonAnalysis", "Analyze"),
            icon: "task-analysis",
            completed: false,
            enabled: false
        },
        {
            id: "logs",
            text: panel.logRunCount > 0 ? qsTranslate("AnalysisLogPanel", "Logs (%1)").arg(panel.logRunCount) : qsTranslate("ImportTask", "Logs"),
            icon: "log",
            completed: false
        }
    ]

    signal meshToolRequested
    signal openProjectRequested
    signal analysisSequenceRequested
    signal processSettingsRequested
    signal logsRequested
    signal resultSelected(string resultId)
    signal materialRequested
    signal newProjectRequested
    signal openImportRequested(string recordId)

    function openPlanTask(taskId) {
        if (taskId === "create-mesh") {
            meshToolRequested();
        } else if (taskId === "analysis-sequence") {
            analysisSequenceRequested();
        } else if (taskId === "process-settings" && panel.processSettingsAvailable) {
            processSettingsRequested();
        } else if (taskId === "material-data") {
            materialRequested();
        } else if (taskId === "logs") {
            logsRequested();
        }
    }

    // Welcome 或空白视口没有对应导入记录，此时仍显示最近导入项。
    readonly property string activePartTitle: importedPartIds.indexOf(activeDocumentId) >= 0 ? activeDocumentTitle : importedPartName

    spacing: 0

    ListView {
        id: projectTree
        objectName: "projectTreeSection"
        Layout.fillWidth: true
        Layout.preferredHeight: Math.max(0, panel.height - (panel.projectOpen ? Theme.borderWidth : 0)) * Theme.projectTreeHeightRatio
        clip: true
        model: panel.importedPartNames
        reuseItems: true
        ScrollBar.vertical: ScrollBar {
            policy: ScrollBar.AlwaysOff
        }

        header: ColumnLayout {
            width: projectTree.width
            spacing: 0

            // Tasks 的空工程入口；工程打开后由工程树和 Plan tasks 接替。
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0
                visible: !panel.projectOpen

                ThemedToolButton {
                    objectName: "openProjectTaskAction"
                    Layout.fillWidth: true
                    text: qsTranslate("IconActionOpenProject", "Open Project")
                    dimText: "…"
                    iconName: "open"
                    preserveIconColors: true
                    iconSize: Theme.iconSizeSmall
                    contentAlignLeft: true
                    contentColor: Theme.colorText
                    contentPadding: Theme.spacingLarge
                    hoverColor: Theme.colorHover
                    onClicked: panel.openProjectRequested()
                }
                ThemedToolButton {
                    objectName: "newProjectTaskAction"
                    Layout.fillWidth: true
                    text: qsTranslate("UiCommonNavigation", "New Project")
                    dimText: "…"
                    iconName: "new"
                    preserveIconColors: true
                    iconSize: Theme.iconSizeSmall
                    contentAlignLeft: true
                    contentColor: Theme.colorText
                    contentPadding: Theme.spacingLarge
                    hoverColor: Theme.colorHover
                    onClicked: panel.newProjectRequested()
                }
            }

            ThemedToolButton {
                id: projectEntry
                objectName: "projectTaskItem"
                Layout.fillWidth: true
                visible: panel.projectOpen
                text: qsTranslate("ProjectTaskItem", "Project '%1'").arg(panel.projectName)
                iconName: "project-file"
                preserveIconColors: true
                iconSize: Theme.iconSizeSmall
                contentAlignLeft: true
                contentColor: Theme.colorText
                contentPadding: Theme.spacingLarge
                hoverColor: Theme.colorHover
                contentItem: RowLayout {
                    spacing: Theme.spacingMedium
                    ThemedIcon {
                        name: projectEntry.iconName
                        iconSize: projectEntry.iconSize
                        preserveSourceColors: true
                        color: projectEntry.contentColor
                    }
                    ThemedLabel {
                        Layout.fillWidth: true
                        text: projectEntry.text
                        textSize: projectEntry.font.pixelSize
                        elide: Text.ElideRight
                    }
                }
            }
        }

        delegate: Rectangle {
            id: importedPartEntry
            objectName: "importedPartEntry"
            required property var modelData
            required property int index

            readonly property string recordId: panel.importedPartIds[index] ?? ""
            readonly property bool isActiveDocument: recordId !== "" && recordId === panel.activeDocumentId

            width: projectTree.width
            height: Theme.controlHeight
            color: isActiveDocument ? Theme.colorSelected : partHover.hovered ? Theme.colorDocumentHover : Theme.colorTransparent

            HoverHandler {
                id: partHover
                acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
            }
            TapHandler {
                id: partTap
                gesturePolicy: TapHandler.ReleaseWithinBounds
                onTapped: panel.openImportRequested(importedPartEntry.recordId)
            }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.spacingLarge * 2
                anchors.rightMargin: Theme.spacingSmall
                spacing: Theme.spacingSmall

                ThemedIcon {
                    name: "stl-file"
                    iconSize: Theme.iconSizeSmall
                    preserveSourceColors: true
                }
                ThemedLabel {
                    Layout.fillWidth: true
                    text: importedPartEntry.modelData
                    textSize: Theme.fontBody
                    elide: Text.ElideRight
                }
            }
        }
    }

    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: Theme.borderWidth
        visible: panel.projectOpen
        color: Theme.colorPanelLine
    }

    ScrollView {
        id: planTaskSection
        objectName: "planTasksSection"
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.preferredHeight: Math.max(0, panel.height - (panel.projectOpen ? Theme.borderWidth : 0)) * (1 - Theme.projectTreeHeightRatio)
        clip: true
        contentWidth: availableWidth
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
        ScrollBar.vertical.policy: ScrollBar.AlwaysOff

        ColumnLayout {
            width: planTaskSection.availableWidth
            spacing: 0
            visible: panel.projectOpen && panel.importedPartAvailable

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.panelToolbarHeight
                color: Theme.colorChrome

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingMedium
                    anchors.rightMargin: Theme.paneCloseSize + Theme.spacingMedium
                    spacing: Theme.spacingSmall

                    ThemedIcon {
                        name: "plan-tasks"
                        iconSize: Theme.iconSizeSmall
                        preserveSourceColors: true
                    }
                    ThemedLabel {
                        Layout.fillWidth: true
                        text: qsTranslate("ProjectTaskItem", "Plan tasks: %1").arg(panel.activePartTitle)
                        textSize: Theme.fontBody
                        elide: Text.ElideRight
                    }
                }
            }

            Repeater {
                model: panel.planTaskItems

                delegate: RowLayout {
                    id: taskRow
                    required property var modelData
                    objectName: "planTaskItem_" + taskRow.modelData.id

                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.spacingMedium
                    Layout.rightMargin: Theme.spacingSmall
                    spacing: Theme.spacingSmall

                    Item {
                        Layout.preferredWidth: Theme.iconSizeSmall
                        Layout.preferredHeight: Theme.iconSizeSmall

                        ThemedIcon {
                            anchors.centerIn: parent
                            visible: taskRow.modelData.id !== "logs" && taskRow.modelData.completed === true
                            name: "status-ok"
                            iconSize: Theme.iconSizeSmall
                            preserveSourceColors: true
                        }
                        CheckIndicator {
                            anchors.centerIn: parent
                            objectName: taskRow.modelData.id === "logs" ? "logsVisibilityIndicator" : ""
                            visible: taskRow.modelData.id === "logs"
                            checked: panel.logsOpen
                        }
                    }

                    ThemedToolButton {
                        Layout.fillWidth: true
                        objectName: "planTaskAction_" + taskRow.modelData.id
                        text: taskRow.modelData.text
                        iconName: taskRow.modelData.icon
                        preserveIconColors: true
                        iconSize: Theme.iconSizeSmall
                        accessibleName: taskRow.modelData.id === "material-data" ? qsTranslate("MaterialDialog", "Select Material") : taskRow.modelData.id === "analysis-sequence" ? qsTranslate("AnalysisSequenceDialog", "Select Analysis Sequence") : taskRow.modelData.id === "create-mesh" ? qsTranslate("ImportTask", "Create Mesh...") : text
                        enabled: taskRow.modelData.enabled !== false
                        contentAlignLeft: true
                        contentColor: Theme.colorText
                        contentPadding: Theme.spacingSmall
                        hoverColor: Theme.colorHover
                        onClicked: panel.openPlanTask(taskRow.modelData.id)
                    }
                }
            }
            AnalysisResultsTree {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.iconSizeSmall + Theme.spacingMedium + Theme.spacingSmall
                Layout.rightMargin: Theme.spacingSmall
                visible: panel.resultRun !== null && panel.resultRun.resultGroups.length > 0
                run: panel.resultRun
                onResultSelected: resultId => panel.resultSelected(resultId)
            }
        }
    }
}

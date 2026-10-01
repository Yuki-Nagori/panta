// 左侧多页签宿主：装配工程任务、工具与 Shared Views。
import QtQuick
import QtQuick.Layouts

PanelSurface {
    id: panel
    property bool meshToolOpen: false
    property alias projectOpen: tasksPage.projectOpen
    property alias projectName: tasksPage.projectName
    property alias importedPartNames: tasksPage.importedPartNames
    property alias importedPartIds: tasksPage.importedPartIds
    property alias activeDocumentId: tasksPage.activeDocumentId
    property alias activeDocumentTitle: tasksPage.activeDocumentTitle
    property alias importedPartAvailable: tasksPage.importedPartAvailable
    property alias importedPartName: tasksPage.importedPartName
    property alias analysisSequenceText: tasksPage.analysisSequenceText
    property alias analysisSequenceId: tasksPage.analysisSequenceId
    property alias materialText: tasksPage.materialText
    property alias materialId: tasksPage.materialId
    property alias fillSettingsConfirmed: tasksPage.fillSettingsConfirmed
    property alias gateLocationSettingsConfirmed: tasksPage.gateLocationSettingsConfirmed
    property alias meshType: tasksPage.meshType
    property alias meshTypes: tasksPage.meshTypes
    property alias logsOpen: tasksPage.logsOpen
    property alias logRunCount: tasksPage.logRunCount
    property alias resultRun: tasksPage.resultRun
    readonly property string activePartTitle: tasksPage.activePartTitle
    signal closeRequested
    signal openProjectRequested
    signal analysisSequenceRequested
    signal processSettingsRequested
    signal logsRequested
    signal resultSelected(string resultId)
    signal materialRequested
    signal newProjectRequested
    signal openImportRequested(string recordId)
    implicitWidth: Theme.leftPanelMinimumWidth
    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        PanelTabBar {
            id: panelTabs
            Layout.fillWidth: true
            rightPadding: Theme.paneCloseSize + 2 * Theme.spacingXSmall
            tabs: [qsTranslate("TaskPanelTitle", "Tasks"), qsTranslate("UiCommonNavigation", "Tools"), qsTranslate("UiCommonNavigation", "Shared Views")]
            onCurrentIndexChanged: panel.meshToolOpen = false
            onTabActivated: panel.meshToolOpen = false
        }
        StackLayout {
            objectName: "sidebarPages"
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: panelTabs.currentIndex
            ProjectTasksPage {
                id: tasksPage
                onMeshToolRequested: {
                    panelTabs.currentIndex = 1;
                    panel.meshToolOpen = true;
                }
                onOpenProjectRequested: panel.openProjectRequested()
                onAnalysisSequenceRequested: panel.analysisSequenceRequested()
                onProcessSettingsRequested: panel.processSettingsRequested()
                onLogsRequested: panel.logsRequested()
                onResultSelected: resultId => panel.resultSelected(resultId)
                onMaterialRequested: panel.materialRequested()
                onNewProjectRequested: panel.newProjectRequested()
                onOpenImportRequested: recordId => panel.openImportRequested(recordId)
            }
            ToolsPage {
                meshToolOpen: panel.meshToolOpen
            }
            SharedViewsPage {}
        }
    }
}

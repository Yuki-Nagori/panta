// 按方案上下文展示运行记录；关闭只改变宿主可见性，不删除传入数据。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

PanelSurface {
    id: panel
    objectName: "analysisLogPanel"

    property var runs: []
    property string contextId: ""
    property string selectedRunId: ""
    property string selectedCategoryId: "analysis"
    readonly property var contextRuns: runs.filter(run => run.contextId === contextId)
    readonly property int selectedRunIndex: contextRuns.findIndex(run => run.id === selectedRunId)
    readonly property var currentRun: selectedRunIndex >= 0 ? contextRuns[selectedRunIndex] : null
    readonly property var categories: currentRun && currentRun.categories.length > 0 ? currentRun.categories : [
        {
            id: "mesh",
            sourceText: "Mesh Log",
            text: ""
        },
        {
            id: "analysis",
            sourceText: "Analysis Log",
            text: ""
        }
    ]
    readonly property int categoryIndex: Math.max(0, categories.findIndex(category => category.id === selectedCategoryId))
    readonly property string currentText: currentRun && currentRun.categories.length > 0 ? categories[categoryIndex].text : qsTr("No analysis logs yet.")
    signal closeRequested

    function selectLatestRun() {
        selectedRunId = contextRuns.length > 0 ? contextRuns[contextRuns.length - 1].id : "";
    }
    onContextIdChanged: selectLatestRun()
    onContextRunsChanged: {
        if (selectedRunIndex < 0)
            selectLatestRun();
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.borderWidth
            color: Theme.colorPanelLine
        }
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.panelToolbarHeight
            color: Theme.colorChrome
            RowLayout {
                anchors.fill: parent
                anchors.rightMargin: Theme.paneCloseSize + 2 * Theme.spacingSmall
                spacing: Theme.spacingSmall
                PanelTabBar {
                    objectName: "analysisLogTabs"
                    Layout.fillWidth: true
                    tabs: panel.categories.map(category => qsTranslate("AnalysisLogCategory", category.sourceText))
                    currentIndex: panel.categoryIndex
                    onTabActivated: index => panel.selectedCategoryId = panel.categories[index].id
                }
                ThemedComboBox {
                    id: runChoice

                    font.pixelSize: Theme.fontSmall
                    objectName: "analysisLogRunChoice"
                    Layout.preferredWidth: Theme.analysisLogRunWidth

                    Accessible.name: qsTr("Analysis run")
                    enabled: panel.contextRuns.length > 0
                    model: panel.contextRuns.map((run, index) => qsTr("Run %1").arg(index + 1))
                    currentIndex: panel.selectedRunIndex
                    displayText: panel.currentRun ? currentText : qsTr("No runs")
                    onActivated: index => panel.selectedRunId = panel.contextRuns[index].id
                }
            }
            PaneCloseButton {
                objectName: "analysisLogClose"
                accessibleName: qsTr("Close logs")
                onCloseRequested: panel.closeRequested()
            }
        }
        ScrollView {
            id: logScroll
            objectName: "analysisLogScroll"
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            TextArea {
                objectName: "analysisLogText"
                text: panel.currentText
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.NoWrap
                font.family: Theme.analysisLogFontFamily
                font.pixelSize: Theme.fontSmall
                color: Theme.colorText
                selectionColor: Theme.colorSelected
                padding: Theme.spacingMedium
                background: Item {}
            }
        }
    }
}

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
                ComboBox {
                    id: runChoice
                    objectName: "analysisLogRunChoice"
                    Layout.preferredWidth: Theme.analysisLogRunWidth
                    implicitHeight: Theme.controlHeight
                    Accessible.name: qsTr("Analysis run")
                    enabled: panel.contextRuns.length > 0
                    model: panel.contextRuns.map((run, index) => qsTr("Run %1").arg(index + 1))
                    currentIndex: panel.selectedRunIndex
                    displayText: panel.currentRun ? currentText : qsTr("No runs")
                    opacity: enabled ? 1 : Theme.disabledOpacity
                    onActivated: index => panel.selectedRunId = panel.contextRuns[index].id
                    leftPadding: Theme.spacingSmall
                    rightPadding: Theme.iconSizeCompact + 2 * Theme.spacingSmall
                    contentItem: ThemedLabel {
                        text: runChoice.displayText
                        textSize: Theme.fontSmall
                        elide: Text.ElideRight
                        verticalAlignment: Text.AlignVCenter
                    }
                    indicator: ThemedIcon {
                        anchors.right: parent.right
                        anchors.rightMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        name: "caret"
                        iconSize: Theme.iconSizeCompact
                    }
                    background: Rectangle {
                        color: Theme.colorPanel
                        radius: Theme.radiusSmall
                        border.color: runChoice.visualFocus ? Theme.colorFocus : Theme.colorPanelLine
                        border.width: runChoice.visualFocus ? Theme.focusBorderWidth : Theme.borderWidth
                    }
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
            contentWidth: availableWidth
            TextArea {
                objectName: "analysisLogText"
                width: logScroll.availableWidth
                text: panel.currentText
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.WrapAnywhere
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

// CaeViewport 原生视口与视口文档页签；工程网格由 Rust 服务快照提供。
// 仅进入启用 Bridge 的构建变体，QML 无法覆盖原生视口表面。
import QtQuick
import Panta.Bridge
import QtQuick.Layouts
import Panta.Visualization

PanelSurface {
    id: panel

    property var meshSource: null
    property var documentSource: panel.meshSource
    property FillingPreviewModel previewModel: null
    property bool reducedMotion: false
    property bool logsOpen: false
    property string logContextId: ""
    property alias logRuns: analysisLogs.runs
    readonly property int logRunCount: analysisLogs.contextRuns.length
    readonly property var selectedLogRun: analysisLogs.currentRun
    signal closeRequested

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            CaeViewport {
                objectName: "caeViewport"
                anchors.fill: parent
                meshSource: panel.meshSource
                playbackTime: panel.previewModel ? panel.previewModel.playbackTime : 0
            }
        }

        AnalysisLogPanel {
            id: analysisLogs
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(Theme.analysisLogHeight, panel.height * Theme.analysisLogMaximumRatio)
            visible: panel.logsOpen && panel.previewModel === null
            contextId: panel.logContextId
            onCloseRequested: panel.logsOpen = false
        }

        FillingPlaybackBar {
            Layout.fillWidth: true
            Layout.margins: Theme.spacingMedium
            previewModel: panel.previewModel
        }

        DocumentTabBar {
            visible: panel.previewModel === null
            objectName: "documentTabBar"
            Layout.fillWidth: true
            documents: panel.documentSource ? panel.documentSource.openDocuments : []
            activeDocumentId: panel.documentSource ? panel.documentSource.activeDocumentId : ""
            reducedMotion: panel.reducedMotion
            onActivateDocument: documentId => panel.documentSource.activateDocument(documentId)
            onCloseDocument: documentId => panel.documentSource.closeDocument(documentId)
            onMoveDocument: (fromIndex, toIndex) => panel.documentSource.moveDocument(fromIndex, toIndex)
        }
    }
}

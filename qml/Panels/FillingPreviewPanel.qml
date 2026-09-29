// 固定算例入口：参数只读，操作经 C++ ViewModel 提交给 Rust 服务。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Panta.Bridge

PanelSurface {
    id: panel
    required property FillingPreviewModel previewModel

    ScrollView {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        contentWidth: availableWidth
        clip: true
        ColumnLayout {
            width: parent.width
            spacing: Theme.spacingLarge
            ThemedLabel {
                text: qsTr("Filling · Default example")
                font.bold: true
            }
            ThemedLabel {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: qsTr("Cover / surface-pair / CPU\nUse the defaults to run the complete filling workflow.")
                textColor: Theme.colorTextMuted
            }
            Repeater {
                model: [qsTr("1  Load example"), qsTr("2  Generate mesh"), qsTr("3  Run filling"), qsTr("4  Replay results")]
                delegate: ThemedLabel {
                    required property int index
                    required property string modelData
                    Layout.fillWidth: true
                    text: (panel.previewModel.step > index ? "✓  " : "") + modelData
                    font.bold: panel.previewModel.step === index
                    textColor: panel.previewModel.step === index ? Theme.colorText : Theme.colorTextMuted
                }
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: Theme.borderWidth
                color: Theme.colorPanelLine
            }
            ThemedLabel {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: qsTr("Fixed inputs\nCover: 202 × 6 × 152 mm\nMaterial: Polyflam RIPP 3625 CS1\nMesh edge: 12 mm\nGate: (102, 3, 82) mm\nMelt / mold: 230 / 40 °C\nFlow: 94.7 cm³/s\nV/P: 99% volume\nPressure limit: 140 MPa\nClamp limit: 350 t\nNon-isothermal · 12 layers")
            }
            ThemedLabel {
                visible: panel.previewModel.triangles > 0
                text: qsTr("Triangles: %1").arg(panel.previewModel.triangles)
            }
            ThemedToolButton {
                objectName: "previewAdvance"
                Layout.fillWidth: true
                contentAlignLeft: true
                borderColor: Theme.colorPanelLine
                highlighted: true
                enabled: !panel.previewModel.busy && panel.previewModel.step < 3
                text: panel.previewModel.step === 0 ? qsTr("Load default example") : panel.previewModel.step === 1 ? qsTr("Generate default mesh") : panel.previewModel.step === 2 ? qsTr("Start filling with defaults") : qsTr("Filling complete")
                onClicked: panel.previewModel.advance()
            }
            ProgressBar {
                Layout.fillWidth: true
                visible: panel.previewModel.busy
                indeterminate: panel.previewModel.progress < 0
                value: Math.max(0, panel.previewModel.progress)
            }
            ThemedLabel {
                Layout.fillWidth: true
                wrapMode: Text.WrapAnywhere
                visible: panel.previewModel.error.length === 0
                text: panel.previewModel.message
            }
            ThemedLabel {
                visible: panel.previewModel.elapsed > 0
                text: qsTr("Elapsed: %1 s").arg(panel.previewModel.elapsed.toFixed(1))
                textColor: Theme.colorTextMuted
            }
            ThemedLabel {
                Layout.fillWidth: true
                visible: panel.previewModel.error.length > 0
                wrapMode: Text.WrapAnywhere
                text: panel.previewModel.error
                textColor: Theme.colorError
            }
            ThemedLabel {
                Layout.fillWidth: true
                visible: panel.previewModel.step === 3
                wrapMode: Text.WordWrap
                text: qsTr("Filled\nFilling time: %1 s\nPeak pressure: %2 MPa").arg(panel.previewModel.duration.toFixed(3)).arg(panel.previewModel.peakPressure.toFixed(3))
                font.bold: true
            }
            RowLayout {
                ThemedToolButton {
                    visible: panel.previewModel.busy
                    text: qsTr("Stop")
                    onClicked: panel.previewModel.cancel()
                }
                ThemedToolButton {
                    enabled: !panel.previewModel.busy
                    text: qsTr("Start over")
                    onClicked: panel.previewModel.reset()
                }
            }
            ThemedLabel {
                visible: panel.previewModel.outputDir.length > 0
                text: qsTr("Run artifacts")
                font.bold: true
            }
            TextEdit {
                Layout.fillWidth: true
                visible: panel.previewModel.outputDir.length > 0
                text: panel.previewModel.outputDir
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.WrapAnywhere
                color: Theme.colorTextMuted
                font.pixelSize: Theme.fontSmall
                Accessible.name: qsTr("Run artifacts path")
            }
        }
    }
}

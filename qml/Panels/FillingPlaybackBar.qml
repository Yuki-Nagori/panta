// 充填时间轴和色标作为 QML 控件显示在 VTK 视口外。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Panta.Bridge

ColumnLayout {
    id: bar
    property FillingPreviewModel previewModel: null

    visible: bar.previewModel !== null
    RowLayout {
        Layout.fillWidth: true
        ThemedToolButton {
            objectName: "previewPlay"
            enabled: bar.previewModel && bar.previewModel.step === 3
            text: bar.previewModel && bar.previewModel.playing ? qsTr("Pause") : qsTr("Play filling")
            onClicked: bar.previewModel.togglePlayback()
        }
        Slider {
            objectName: "previewTimeline"
            Layout.fillWidth: true
            enabled: bar.previewModel && bar.previewModel.step === 3
            from: 0
            to: bar.previewModel ? bar.previewModel.duration : 1
            value: bar.previewModel ? bar.previewModel.playbackTime : 0
            onMoved: bar.previewModel.playbackTime = value
            Accessible.name: qsTr("Filling time")
        }
        ThemedLabel {
            text: bar.previewModel ? qsTr("%1 / %2 s").arg(bar.previewModel.playbackTime.toFixed(3)).arg(bar.previewModel.duration.toFixed(3)) : ""
        }
    }
    RowLayout {
        Layout.fillWidth: true
        ThemedLabel {
            text: qsTr("Arrival time · 0 s")
        }
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Theme.spacingMedium
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop {
                    position: 0
                    color: Theme.colorFillEarly
                }
                GradientStop {
                    position: 0.333
                    color: Theme.colorFillMidEarly
                }
                GradientStop {
                    position: 0.667
                    color: Theme.colorFillMidLate
                }
                GradientStop {
                    position: 1
                    color: Theme.colorFillLate
                }
            }
        }
        ThemedLabel {
            text: bar.previewModel ? qsTr("%1 s · gray: unfilled").arg(bar.previewModel.duration.toFixed(3)) : ""
        }
    }
}

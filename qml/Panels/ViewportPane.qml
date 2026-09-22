// 中央视口面板（复刻件 .vtk-pane）：CaeViewport 宿主与底部视图页签。
// 任务 007 的原生 surface 宿主，VTK 不进入 Qt Quick scenegraph；视图页签
// 暂为视觉参考，未接业务命令。依赖 Panta.Visualization，仅进入启用 Bridge
// 的模块变体（qml/CMakeLists.txt 剔除逻辑）。
import QtQuick
import QtQuick.Layouts
import Panta.Visualization

PanelSurface {
    id: panel

    signal closeRequested

    PaneCloseButton {
        onCloseRequested: panel.closeRequested()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Item {
            id: viewportArea

            Layout.fillWidth: true
            Layout.fillHeight: true

            CaeViewport {
                objectName: "caeViewport"
                anchors.fill: parent
            }
        }

        PanelTabBar {
            Layout.fillWidth: true
            edge: Qt.BottomEdge
            tabs: [qsTr("Model"), qsTr("Mesh"), qsTr("Results")]
        }
    }
}

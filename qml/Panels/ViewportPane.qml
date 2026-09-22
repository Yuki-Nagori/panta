// CaeViewport 原生视口与视图页签；页签暂未接业务命令。
// 仅进入启用 Bridge 的构建变体，QML 无法覆盖原生视口表面。
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

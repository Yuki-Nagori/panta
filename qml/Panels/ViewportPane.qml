// CaeViewport 原生视口与视图页签；工程内复制的 STL 路径通过 meshPath 注入。
// 仅进入启用 Bridge 的构建变体，QML 无法覆盖原生视口表面。
import QtQuick
import QtQuick.Layouts
import Panta.Visualization

PanelSurface {
    id: panel

    property string meshPath: ""
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
                meshPath: panel.meshPath
            }
        }

        PanelTabBar {
            Layout.fillWidth: true
            edge: Qt.BottomEdge
            tabs: [qsTr("Model"), qsTranslate("UiCommonModeling", "Mesh"), qsTranslate("UiCommonResults", "Results")]
        }
    }
}

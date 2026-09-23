// CaeViewport 原生视口与视图页签；工程网格由 Rust 服务快照提供。
// 仅进入启用 Bridge 的构建变体，QML 无法覆盖原生视口表面。
import QtQuick
import QtQuick.Layouts
import Panta.Visualization

PanelSurface {
    id: panel

    property var meshSource: null
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
            }
        }

        PanelTabBar {
            Layout.fillWidth: true
            edge: Qt.BottomEdge
            tabs: [qsTr("Model"), qsTranslate("UiCommonModeling", "Mesh"), qsTranslate("UiCommonResults", "Results")]
        }
    }
}

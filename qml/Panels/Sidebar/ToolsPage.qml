// 工具页装配当前网格工具与使用说明。
import QtQuick

Item {
    id: page
    property bool meshToolOpen: false

    objectName: "toolsPage"

    InformationPanel {
        objectName: "toolsInformation"
        anchors.fill: parent
        visible: !page.meshToolOpen
        titleText: qsTranslate("TaskPanelInformation", "Information")
        messages: [qsTranslate("TaskPanelToolsHelp", "Use the tools above to access each tool."), qsTranslate("TaskPanelToolsHelp", "Open a tool's help to learn how to use it."), qsTranslate("TaskPanelToolsHelp", "Hold Ctrl while clicking to select multiple entities.")]
    }

    MeshToolPanel {
        anchors.fill: parent
        visible: page.meshToolOpen
    }
}

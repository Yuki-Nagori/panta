// Shared Views 的信息入口。
import QtQuick

Item {
    objectName: "sharedViewsPage"

    InformationPanel {
        objectName: "sharedViewsInformation"
        anchors.fill: parent
        titleText: qsTranslate("TaskPanelInformation", "Information")
        messages: [qsTranslate("TaskPanelInformation", "Access Shared Views from the Shared panel on the Home tab.")]
    }
}

// 分析序列的候选选择；只维护显示状态，确认由窗口处理。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: panel
    property alias selectedIndex: sequenceList.currentIndex
    property var sequences: []
    readonly property string selectedId: sequences[selectedIndex]?.id ?? ""
    readonly property string selectedText: sequences[selectedIndex] ? qsTranslate("AnalysisSequence", sequences[selectedIndex].sourceText) : ""
    function focusSelection() {
        sequenceList.forceActiveFocus();
        sequenceList.positionViewAtIndex(selectedIndex, ListView.Contain);
    }

    spacing: Theme.spacingMedium

    Rectangle {
        Layout.fillWidth: true
        Layout.fillHeight: true
        color: Theme.colorPanel
        border.color: Theme.colorPanelLine
        border.width: Theme.borderWidth

        ListView {
            id: sequenceList
            objectName: "analysisSequenceList"
            anchors.fill: parent
            anchors.margins: Theme.borderWidth
            clip: true
            model: panel.sequences
            currentIndex: 0
            keyNavigationEnabled: true
            focus: true
            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
            }
            delegate: ThemedToolButton {
                id: sequenceRow
                required property var modelData
                required property int index
                width: sequenceList.width
                text: qsTranslate("AnalysisSequence", modelData.sourceText)
                contentAlignLeft: true
                contentPadding: Theme.spacingMedium
                contentColor: Theme.colorText
                hoverColor: Theme.colorHover
                background: Rectangle {
                    color: sequenceRow.index === panel.selectedIndex ? Theme.colorSelected : sequenceRow.hovered ? Theme.colorHover : Theme.colorTransparent
                }
                onClicked: {
                    panel.selectedIndex = index;
                    panel.focusSelection();
                }
            }
        }
    }
    ThemedLabel {
        Layout.fillWidth: true
        text: panel.selectedText
        font.weight: Font.DemiBold
        wrapMode: Text.Wrap
    }
}

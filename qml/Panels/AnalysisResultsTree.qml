// 只展示运行产出的目录；当前结果 ID 统一控制跨分组的方框单选。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ColumnLayout {
    id: tree
    objectName: "analysisResultsTree"

    property var run: null
    readonly property var groups: run ? run.resultGroups : []
    property var selections: ({})
    property string selectedResultId: ""
    property bool expanded: true
    signal resultSelected(string resultId)
    spacing: 0

    function selectionKey() {
        return run ? JSON.stringify([run.contextId, run.id]) : "";
    }
    function restoreSelection() {
        const ids = [];
        if (run) {
            for (const group of run.resultGroups) {
                for (const result of group.results)
                    ids.push(result.id);
            }
        }
        const remembered = selections?.[selectionKey()];
        selectedResultId = ids.includes(remembered) ? remembered : ids[0] ?? "";
    }
    onRunChanged: restoreSelection()
    Component.onCompleted: restoreSelection()

    ButtonGroup {
        id: resultChoices
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.spacingSmall
        ThemedIcon {
            name: "caret"
            iconSize: Theme.iconSizeCompact
            rotation: tree.expanded ? 0 : -90
        }
        ThemedToolButton {
            objectName: "analysisResultsToggle"
            Layout.fillWidth: true
            iconName: "ribbon-results"
            preserveIconColors: true
            iconSize: Theme.iconSizeSmall
            text: qsTranslate("UiCommonResults", "Results")
            contentAlignLeft: true
            contentColor: Theme.colorText
            onClicked: tree.expanded = !tree.expanded
        }
    }
    Repeater {
        model: tree.groups
        delegate: ColumnLayout {
            id: group
            required property var modelData
            property bool expanded: true
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMedium
            spacing: 0
            visible: tree.expanded
            RowLayout {
                Layout.fillWidth: true
                visible: group.modelData.sourceText.length > 0
                spacing: Theme.spacingSmall
                ThemedIcon {
                    name: "caret"
                    iconSize: Theme.iconSizeCompact
                    rotation: group.expanded ? 0 : -90
                }
                ThemedToolButton {
                    Layout.fillWidth: true
                    text: qsTranslate("AnalysisResult", group.modelData.sourceText)
                    iconName: "project-folder"
                    iconSize: Theme.iconSizeSmall
                    preserveIconColors: true
                    contentAlignLeft: true
                    contentColor: Theme.colorText
                    onClicked: group.expanded = !group.expanded
                }
            }
            Repeater {
                model: group.modelData.results
                delegate: RadioButton {
                    id: choice
                    required property var modelData
                    objectName: "analysisResult_" + modelData.id
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.spacingMedium
                    visible: group.expanded || group.modelData.sourceText.length === 0
                    ButtonGroup.group: resultChoices
                    checked: tree.selectedResultId === modelData.id
                    text: qsTranslate("AnalysisResult", modelData.sourceText)
                    implicitHeight: Theme.controlHeight
                    padding: Theme.spacingTiny
                    spacing: Theme.spacingSmall
                    hoverEnabled: true
                    onClicked: {
                        tree.selectedResultId = modelData.id;
                        tree.selections[tree.selectionKey()] = modelData.id;
                        tree.resultSelected(modelData.id);
                    }
                    indicator: CheckIndicator {
                        y: (choice.height - height) / 2
                        checked: choice.checked
                        focused: choice.visualFocus
                    }
                    contentItem: ThemedLabel {
                        leftPadding: choice.indicator.width + choice.spacing
                        text: choice.text
                        textSize: Theme.fontBody
                        font.weight: choice.checked ? Font.DemiBold : Font.Normal
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideRight
                    }
                    background: Rectangle {
                        color: choice.hovered || choice.visualFocus ? Theme.colorHover : Theme.colorTransparent
                        radius: Theme.radiusSmall
                    }
                }
            }
        }
    }
}

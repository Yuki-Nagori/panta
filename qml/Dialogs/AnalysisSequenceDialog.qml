// 分析序列弹窗；已确认值和候选值分离，取消不会改变任务行。
import QtQuick
import QtQuick.Layouts

Window {
    id: dialog
    objectName: "analysisSequenceDialog"
    property Window ownerWindow
    property var sequences: []
    property var planSettings: ({})
    property string errorText: ""
    // 打开时冻结命令目标；后台或程序化工程切换不能把确认写入另一个方案。
    property var openedSettings: ({})
    signal selectionRequested(string projectPath, double revision, string importId, string sequenceId)

    width: Theme.analysisSequenceDialogWidth
    height: Theme.analysisSequenceDialogHeight
    minimumWidth: Theme.analysisSequenceDialogMinimumWidth
    minimumHeight: Theme.analysisSequenceDialogMinimumHeight
    color: Theme.colorTransparent
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: qsTranslate("AnalysisSequenceDialog", "Select Analysis Sequence")
    transientParent: ownerWindow

    function open() {
        openedSettings = planSettings;
        sequencePanel.selectedIndex = sequences.findIndex(entry => entry.id === openedSettings.sequenceId);
        if (sequencePanel.selectedIndex < 0 || !openedSettings.importId)
            return;
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        show();
        requestActivate();
        sequencePanel.focusSelection();
    }

    function acceptSelection() {
        if (sequencePanel.selectedId.length > 0 && openedSettings.importId)
            selectionRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, sequencePanel.selectedId);
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.colorPanel
        ColumnLayout {
            anchors.fill: parent
            spacing: 0
            DialogTitleBar {
                Layout.fillWidth: true
                window: dialog
                caption: dialog.title
                onCloseRequested: dialog.close()
            }
            ThemedLabel {
                Layout.fillWidth: true
                Layout.margins: Theme.spacingLarge
                visible: dialog.errorText.length > 0
                text: dialog.errorText
                textColor: Theme.colorError
                wrapMode: Text.Wrap
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.margins: Theme.spacingLarge
                spacing: Theme.spacingLarge
                AnalysisSequencePanel {
                    id: sequencePanel
                    objectName: "analysisSequencePanel"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    sequences: dialog.sequences
                }
                ColumnLayout {
                    Layout.fillHeight: true
                    spacing: Theme.spacingSmall
                    ThemedToolButton {
                        objectName: "analysisSequenceAccept"
                        Layout.preferredWidth: Theme.tabSegmentWidth
                        text: qsTranslate("DialogAction", "OK")
                        enabled: sequencePanel.selectedId.length > 0
                        primaryAction: true
                        contentColor: Theme.colorText
                        borderColor: Theme.colorDialogPrimaryBorder
                        hoverColor: Theme.colorHover
                        clickAction: () => dialog.acceptSelection()
                    }
                    ThemedToolButton {
                        objectName: "analysisSequenceCancel"
                        Layout.preferredWidth: Theme.tabSegmentWidth
                        text: qsTranslate("DialogAction", "Cancel")
                        contentColor: Theme.colorText
                        borderColor: Theme.colorPanelLine
                        hoverColor: Theme.colorHover
                        clickAction: () => dialog.close()
                    }
                    Item {
                        Layout.fillHeight: true
                    }
                    ThemedToolButton {
                        Layout.preferredWidth: Theme.tabSegmentWidth
                        text: qsTranslate("AnalysisSequenceDialog", "More...")

                        contentColor: Theme.colorText
                        borderColor: Theme.colorPanelLine
                    }
                }
            }
        }
    }
    Shortcut {
        sequence: "Esc"
        onActivated: dialog.close()
    }
    Shortcut {
        sequence: "Return"
        onActivated: dialog.acceptSelection()
    }
}

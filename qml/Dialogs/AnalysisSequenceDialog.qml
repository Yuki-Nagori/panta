// 分析序列弹窗；已确认值和候选值分离，取消不会改变任务行。
import QtQuick
import QtQuick.Layouts

DialogWindow {
    id: dialog
    objectName: "analysisSequenceDialog"
    property var sequences: []
    property var planSettings: ({})
    property string errorText: ""
    property bool saving: false
    // 打开时冻结命令目标；后台或程序化工程切换不能把确认写入另一个方案。
    property var openedSettings: ({})
    signal selectionRequested(string projectPath, double revision, string importId, string sequenceId)

    width: Theme.analysisSequenceDialogWidth
    height: Theme.analysisSequenceDialogHeight
    minimumWidth: Theme.analysisSequenceDialogMinimumWidth
    minimumHeight: Theme.analysisSequenceDialogMinimumHeight
    title: qsTranslate("AnalysisSequenceDialog", "Select Analysis Sequence")
    onClosing: event => {
        if (saving)
            event.accepted = false;
    }

    function open() {
        if (saving)
            return;
        openedSettings = planSettings;
        sequencePanel.selectedIndex = sequences.findIndex(entry => entry.id === openedSettings.sequenceId);
        if (sequencePanel.selectedIndex < 0 || !openedSettings.importId)
            return;
        centerOnOwner();
        show();
        requestActivate();
        sequencePanel.focusSelection();
    }

    function acceptSelection() {
        if (visible && !saving && sequencePanel.selectedId.length > 0 && openedSettings.importId)
            selectionRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, sequencePanel.selectedId);
    }

    DialogFrame {
        window: dialog
        closeEnabled: !dialog.saving

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
                enabled: !dialog.saving
            }
            ColumnLayout {
                Layout.fillHeight: true
                spacing: Theme.spacingSmall
                ThemedButton {
                    objectName: "analysisSequenceAccept"

                    text: qsTranslate("DialogAction", "OK")
                    enabled: !dialog.saving && sequencePanel.selectedId.length > 0
                    primaryAction: true
                    contentColor: Theme.colorText
                    hoverColor: Theme.colorHover
                    onClicked: dialog.acceptSelection()
                }
                ThemedButton {
                    objectName: "analysisSequenceCancel"
                    enabled: !dialog.saving

                    text: qsTranslate("DialogAction", "Cancel")
                    contentColor: Theme.colorText
                    borderColor: Theme.colorPanelLine
                    hoverColor: Theme.colorHover
                    onClicked: dialog.close()
                }
                Item {
                    Layout.fillHeight: true
                }
                ThemedButton {
                    text: qsTranslate("AnalysisSequenceDialog", "More...")
                    enabled: false
                    contentColor: Theme.colorText
                    borderColor: Theme.colorPanelLine
                }
            }
        }
    }
    Shortcut {
        enabled: dialog.visible && !dialog.saving
        sequence: "Esc"
        onActivated: dialog.close()
    }
    Shortcut {
        enabled: dialog.visible && !dialog.saving
        sequence: "Return"
        onActivated: dialog.acceptSelection()
    }
}

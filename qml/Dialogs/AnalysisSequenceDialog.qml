// 分析序列弹窗；已确认值和候选值分离，取消不会改变任务行。
import QtQuick
import QtQuick.Layouts

DialogWindow {
    id: dialog
    objectName: "analysisSequenceDialog"
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
    title: qsTranslate("AnalysisSequenceDialog", "Select Analysis Sequence")

    function open() {
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
        if (sequencePanel.selectedId.length > 0 && openedSettings.importId)
            selectionRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, sequencePanel.selectedId);
    }

    DialogFrame {
        window: dialog
        closeEnabled: true

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
                ThemedButton {
                    objectName: "analysisSequenceAccept"

                    text: qsTranslate("DialogAction", "OK")
                    enabled: sequencePanel.selectedId.length > 0
                    primaryAction: true
                    contentColor: Theme.colorText
                    hoverColor: Theme.colorHover
                    onClicked: dialog.acceptSelection()
                }
                ThemedButton {
                    objectName: "analysisSequenceCancel"

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
        sequence: "Esc"
        onActivated: dialog.close()
    }
    Shortcut {
        sequence: "Return"
        onActivated: dialog.acceptSelection()
    }
}

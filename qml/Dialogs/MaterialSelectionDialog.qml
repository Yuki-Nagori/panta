// 材料确认窗口；提交前可放弃候选，后台写入期间保持窗口和目标有效。
import QtQuick
import QtQuick.Layouts

DialogWindow {
    id: dialog
    objectName: "materialSelectionDialog"
    property var material: ({})
    property var planSettings: ({})
    property var openedSettings: ({})
    property string errorText: ""
    property bool saving: false
    readonly property bool canConfirm: !saving && !!material.id && !!openedSettings.importId
    signal selectionRequested(string projectPath, double revision, string importId, string materialId)
    width: Theme.materialDialogWidth
    height: Theme.materialDialogHeight
    minimumWidth: Theme.materialDialogMinimumWidth
    minimumHeight: Theme.materialDialogMinimumHeight
    title: qsTranslate("MaterialDialog", "Select Material")
    onClosing: event => {
        if (saving)
            event.accepted = false;
    }

    function open() {
        if (saving || !material.id || !planSettings.importId)
            return;
        openedSettings = planSettings;
        materialPanel.reset();
        centerOnOwner();
        show();
        requestActivate();
    }

    function acceptSelection() {
        if (visible && canConfirm)
            selectionRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, material.id);
    }

    DialogFrame {
        window: dialog
        closeEnabled: !dialog.saving

        ThemedLabel {
            Layout.fillWidth: true
            Layout.margins: Theme.spacingLarge
            text: dialog.saving ? qsTranslate("MaterialDialog", "Saving material...") : dialog.errorText
            textColor: dialog.saving ? Theme.colorTextMuted : Theme.colorError
            visible: text.length > 0
            wrapMode: Text.Wrap
        }
        MaterialSelectionPanel {
            id: materialPanel
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: Theme.spacingLarge
            material: dialog.material
            enabled: !dialog.saving
        }
        DialogButtonRow {
            Layout.fillWidth: true
            Layout.margins: Theme.spacingLarge
            spacing: Theme.spacingSmall

            ThemedButton {
                objectName: "materialAccept"

                text: qsTranslate("DialogAction", "OK")
                enabled: dialog.canConfirm
                primaryAction: true
                contentColor: Theme.colorText
                onClicked: dialog.acceptSelection()
            }
            ThemedButton {
                objectName: "materialCancel"
                enabled: !dialog.saving

                text: qsTranslate("DialogAction", "Cancel")
                contentColor: Theme.colorText
                borderColor: Theme.colorPanelLine
                onClicked: dialog.close()
            }
            ThemedButton {
                text: qsTranslate("UiCommonHelp", "Help")
                contentColor: Theme.colorText
                borderColor: Theme.colorPanelLine
                enabled: false
            }
        }
    }
    Shortcut {
        enabled: dialog.visible && !dialog.saving
        sequence: "Esc"
        onActivated: dialog.close()
    }
    Shortcut {
        enabled: dialog.visible && dialog.canConfirm
        sequence: "Return"
        onActivated: dialog.acceptSelection()
    }
}

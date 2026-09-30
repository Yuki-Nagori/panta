// 材料确认窗口；冻结工程目标，取消和关闭不提交候选值。
import QtQuick
import QtQuick.Layouts

Window {
    id: dialog
    objectName: "materialSelectionDialog"
    property Window ownerWindow
    property var material: ({})
    property var planSettings: ({})
    property var openedSettings: ({})
    property string errorText: ""
    readonly property bool canConfirm: !!material.id && !!openedSettings.importId
    signal selectionRequested(string projectPath, double revision, string importId, string materialId)
    width: Theme.materialDialogWidth
    height: Theme.materialDialogHeight
    minimumWidth: Theme.materialDialogMinimumWidth
    minimumHeight: Theme.materialDialogMinimumHeight
    color: Theme.colorTransparent
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: qsTranslate("MaterialDialog", "Select Material")
    transientParent: ownerWindow

    function open() {
        if (!material.id || !planSettings.importId)
            return;
        openedSettings = planSettings;
        materialPanel.reset();
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        show();
        requestActivate();
    }

    function acceptSelection() {
        if (visible && canConfirm)
            selectionRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, material.id);
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
                text: dialog.errorText
                textColor: Theme.colorError
                visible: text.length > 0
                wrapMode: Text.Wrap
            }
            MaterialSelectionPanel {
                id: materialPanel
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.margins: Theme.spacingLarge
                material: dialog.material
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.margins: Theme.spacingLarge
                spacing: Theme.spacingSmall
                Item {
                    Layout.fillWidth: true
                }
                ThemedToolButton {
                    objectName: "materialAccept"
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    text: qsTranslate("DialogAction", "OK")
                    enabled: dialog.canConfirm
                    primaryAction: true
                    contentColor: Theme.colorText
                    borderColor: Theme.colorDialogPrimaryBorder
                    onClicked: dialog.acceptSelection()
                }
                ThemedToolButton {
                    objectName: "materialCancel"
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    text: qsTranslate("DialogAction", "Cancel")
                    contentColor: Theme.colorText
                    borderColor: Theme.colorPanelLine
                    onClicked: dialog.close()
                }
                ThemedToolButton {
                    Layout.preferredWidth: Theme.tabSegmentWidth
                    text: qsTranslate("UiCommonHelp", "Help")
                    contentColor: Theme.colorText
                    borderColor: Theme.colorPanelLine
                    enabled: false
                }
            }
        }
    }
    Shortcut {
        enabled: dialog.visible
        sequence: "Esc"
        onActivated: dialog.close()
    }
    Shortcut {
        enabled: dialog.visible && dialog.canConfirm
        sequence: "Return"
        onActivated: dialog.acceptSelection()
    }
}

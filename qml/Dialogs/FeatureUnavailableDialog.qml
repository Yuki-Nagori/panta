// 原生模态窗口可覆盖 VTK surface，并返回到发起提示的工艺弹窗。
import QtQuick
import QtQuick.Layouts

Window {
    id: dialog
    objectName: "featureUnavailableDialog"
    property Window ownerWindow
    width: Theme.controlHeight * 16
    height: Theme.controlHeight * 6
    color: Theme.colorPanel
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: "Coming soon"
    transientParent: ownerWindow

    function open() {
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        show();
        requestActivate();
        acceptButton.forceActiveFocus();
    }

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
            objectName: "featureUnavailableMessage"
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: Theme.spacingLarge
            text: "This feature is not available yet. Stay tuned."
            wrapMode: Text.Wrap
            verticalAlignment: Text.AlignVCenter
        }
        ThemedToolButton {
            id: acceptButton
            objectName: "featureUnavailableAccept"
            Layout.alignment: Qt.AlignRight
            Layout.margins: Theme.spacingLarge
            text: "OK"
            primaryAction: true
            contentColor: Theme.colorText
            clickAction: () => dialog.close()
        }
    }
    Shortcut {
        sequence: "Escape"
        enabled: dialog.visible
        onActivated: dialog.close()
    }
}

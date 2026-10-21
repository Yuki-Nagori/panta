// 不启用 Panta.Bridge 时的最小 Shell；用于资源/依赖裁剪和启动诊断。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: minimalShell
    minimumWidth: Theme.windowMinimumWidth
    minimumHeight: Theme.windowMinimumHeight
    property alias settingsRouter: settingsPageRouter
    visible: true
    visibility: Window.Maximized
    color: Theme.colorBackground

    menuBar: Qt.platform.os === "osx" ? platformMenuBar : null

    PlatformMenuBar {
        id: platformMenuBar
        objectName: "platformApplicationMenuBar"
        visible: Qt.platform.os === "osx"
        hostWindow: minimalShell
        projectCommandsEnabled: false
        onSettingsRequested: settingsPageRouter.openPage("general")
        onQuitRequested: minimalShell.close()
    }

    SettingsPageRouter {
        id: settingsPageRouter
        objectName: "settingsPageRouter"
        ownerWindow: minimalShell
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium

        TopChromePanel {
            objectName: "titleChromePanel"
            Layout.fillWidth: true
            Layout.preferredHeight: implicitHeight
            caption: ""
            projectCommandsEnabled: false
            onSettingsRequested: settingsPageRouter.openPage("general")
            onExitRequested: minimalShell.close()
        }

        ThemedLabel {
            objectName: "shellCaption"
            Layout.fillWidth: true
            text: qsTr("panta — Minimal Shell (Bridge disabled)")
            textSize: Theme.fontTitle
        }

        PlaceholderPanel {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}

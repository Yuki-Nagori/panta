// macOS 桌面应用菜单；其他平台由后台面板承载 Settings 入口。
import QtQuick
import QtQuick.Controls

Menu {
    id: applicationMenu

    signal settingsRequested
    signal quitRequested

    title: qsTranslate("ApplicationMenu", "panta")

    MenuItem {
        objectName: "applicationAboutAction"
        text: qsTranslate("ApplicationMenu", "About panta")
        enabled: false
    }

    MenuSeparator {}

    MenuItem {
        objectName: "applicationSettingsAction"
        text: qsTranslate("ApplicationMenu", "Settings…")
        onTriggered: applicationMenu.settingsRequested()
    }

    MenuSeparator {}

    MenuItem {
        objectName: "applicationQuitAction"
        text: qsTranslate("ApplicationMenu", "Quit panta")
        onTriggered: applicationMenu.quitRequested()
    }
}

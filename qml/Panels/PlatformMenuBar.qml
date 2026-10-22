// macOS 原生菜单栏；平台菜单由 Qt 接管绘制，命令通过信号交给应用宿主。
import QtQuick
import QtQuick.Controls

MenuBar {
    id: platformMenuBar

    objectName: "platformApplicationMenuBar"
    required property Window hostWindow
    property bool projectCommandsEnabled: true
    enabled: Qt.platform.os === "osx"

    signal newProjectRequested
    signal openProjectRequested
    signal settingsRequested
    signal quitRequested

    Action {
        id: undoAction
        text: qsTranslate("ApplicationMenu", "Undo")
        shortcut: StandardKey.Undo
        enabled: EditCommandRouter.canUndo
        onTriggered: EditCommandRouter.undo()
    }
    Action {
        id: redoAction
        text: qsTranslate("ApplicationMenu", "Redo")
        shortcut: StandardKey.Redo
        enabled: EditCommandRouter.canRedo
        onTriggered: EditCommandRouter.redo()
    }
    Action {
        id: cutAction
        text: qsTranslate("ApplicationMenu", "Cut")
        shortcut: StandardKey.Cut
        enabled: EditCommandRouter.canCut
        onTriggered: EditCommandRouter.cut()
    }
    Action {
        id: copyAction
        text: qsTranslate("ApplicationMenu", "Copy")
        shortcut: StandardKey.Copy
        enabled: EditCommandRouter.canCopy
        onTriggered: EditCommandRouter.copy()
    }
    Action {
        id: pasteAction
        text: qsTranslate("ApplicationMenu", "Paste")
        shortcut: StandardKey.Paste
        enabled: EditCommandRouter.canPaste
        onTriggered: EditCommandRouter.paste()
    }
    Action {
        id: selectAllAction
        text: qsTranslate("ApplicationMenu", "Select All")
        shortcut: StandardKey.SelectAll
        enabled: EditCommandRouter.canSelectAll
        onTriggered: EditCommandRouter.selectAll()
    }
    Action {
        id: minimizeAction
        text: qsTranslate("ApplicationMenu", "Minimize")
        shortcut: "Ctrl+M"
        onTriggered: platformMenuBar.hostWindow.showMinimized()
    }

    Menu {
        objectName: "platformFileMenu"
        title: qsTranslate("ApplicationMenu", "File")

        MenuItem {
            objectName: "platformNewProjectAction"
            text: qsTranslate("ApplicationMenu", "New Project…")
            enabled: platformMenuBar.projectCommandsEnabled
            onTriggered: platformMenuBar.newProjectRequested()
        }
        MenuItem {
            objectName: "platformOpenProjectAction"
            text: qsTranslate("ApplicationMenu", "Open Project…")
            enabled: platformMenuBar.projectCommandsEnabled
            onTriggered: platformMenuBar.openProjectRequested()
        }
        MenuSeparator {}
        MenuItem {
            text: qsTranslate("ApplicationMenu", "Save")
            enabled: false
        }
        MenuItem {
            text: qsTranslate("ApplicationMenu", "Close")
            enabled: false
        }

        MenuSeparator {}
        MenuItem {
            objectName: "applicationAboutAction"
            text: qsTranslate("ApplicationMenu", "About panta")
            enabled: false
        }
        MenuItem {
            objectName: "applicationSettingsAction"
            text: qsTranslate("ApplicationMenu", "Settings…")
            onTriggered: platformMenuBar.settingsRequested()
        }
        MenuItem {
            objectName: "applicationQuitAction"
            text: qsTranslate("ApplicationMenu", "Quit panta")
            onTriggered: platformMenuBar.quitRequested()
        }
    }

    Menu {
        objectName: "platformEditMenu"
        title: qsTranslate("ApplicationMenu", "Edit")

        MenuItem {
            action: undoAction
        }
        MenuItem {
            action: redoAction
        }
        MenuSeparator {}
        MenuItem {
            action: cutAction
        }
        MenuItem {
            action: copyAction
        }
        MenuItem {
            action: pasteAction
        }
        MenuSeparator {}
        MenuItem {
            action: selectAllAction
        }
    }

    Menu {
        objectName: "platformWindowMenu"
        title: qsTranslate("ApplicationMenu", "Window")

        MenuItem {
            action: minimizeAction
        }
        MenuItem {
            text: qsTranslate("ApplicationMenu", "Zoom")
            onTriggered: platformMenuBar.hostWindow.visibility === Window.Maximized ? platformMenuBar.hostWindow.showNormal() : platformMenuBar.hostWindow.showMaximized()
        }
    }
}

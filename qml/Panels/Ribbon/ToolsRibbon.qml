// 工程 Tools 页签展示独立工具分组；尚未接入服务的操作全部禁用。
pragma ComponentBehavior: Bound

import QtQuick

Row {
    id: toolsRibbon

    component DisabledTile: RibbonTile {
        property string actionKey: ""
        objectName: "ribbon-" + actionKey
        enabled: false
    }

    component MacroSlotButton: ThemedToolButton {
        required property int slotNumber
        width: Theme.ribbonMacroSlotSize
        controlHeight: Theme.ribbonMacroSlotSize
        contentPadding: 0
        cornerRadius: width / 2
        borderColor: Theme.colorPanelLine
        font.pixelSize: Theme.fontSmall
        text: String(slotNumber)
        accessibleName: qsTranslate("RibbonActionMacroSlot", "Macro %1").arg(slotNumber)
        enabled: false
        objectName: "ribbon-macro-" + slotNumber
    }

    RibbonGroup {
        title: qsTranslate("RibbonGroupToolsDatabase", "Database")
        DisabledTile {
            actionKey: "database-search"
            iconName: "tools-database-search"
            text: qsTranslate("RibbonActionDatabaseSearch", "Search")
        }
        DisabledTile {
            actionKey: "database-new"
            iconName: "tools-database-new"
            text: qsTranslate("RibbonActionDatabaseNew", "New")
        }
        DisabledTile {
            actionKey: "database-edit"
            iconName: "tools-database-edit"
            text: qsTranslate("RibbonActionDatabaseEdit", "Edit")
        }
    }
    RibbonGroup {
        title: qsTranslate("RibbonGroupToolsAutomation", "Automation")
        DisabledTile {
            actionKey: "record-macro"
            iconName: "tools-record"
            text: qsTranslate("RibbonActionRecordMacro", "Record")
        }
        DisabledTile {
            actionKey: "stop-recording"
            iconName: "tools-stop"
            text: qsTranslate("RibbonActionStopRecording", "Stop\nRecording")
        }
        DisabledTile {
            actionKey: "play-macro"
            iconName: "tools-play"
            text: qsTranslate("RibbonActionPlayMacro", "Play\nMacro")
        }
        DisabledTile {
            actionKey: "command-line"
            iconName: "tools-command-line"
            text: qsTranslate("RibbonActionCommandLine", "Command\nLine")
        }
    }
    RibbonGroup {
        title: qsTranslate("RibbonGroupToolsMacro", "Specify Macro")
        width: Theme.ribbonMacroGroupWidth
        RibbonTile {
            objectName: "ribbon-specify-macro"
            iconName: "tools-specify-macro"
            text: qsTranslate("RibbonActionSpecifyMacro", "Specify\nMacro")
            enabled: false
        }
        Grid {
            anchors.verticalCenter: parent.verticalCenter
            columns: 5
            spacing: Theme.ribbonMacroSlotSpacing
            Repeater {
                model: [1, 3, 5, 7, 9, 2, 4, 6, 8, 10]
                delegate: MacroSlotButton {
                    required property int modelData
                    slotNumber: modelData
                }
            }
        }
    }
    RibbonGroup {
        title: qsTranslate("RibbonGroupToolsOptions", "Options")
        DisabledTile {
            actionKey: "application-options"
            iconName: "tools-options"
            text: qsTranslate("RibbonActionApplicationOptions", "Application\nOptions")
        }
    }
    RibbonGroup {
        title: qsTranslate("RibbonGroupTaskKeyManager", "Task Key Manager")
        DisabledTile {
            actionKey: "key-manager"
            iconName: "tools-key-manager"
            text: qsTranslate("RibbonActionKeyManager", "Key Manager")
        }
    }
    RibbonGroup {
        title: qsTranslate("RibbonGroupToolsPerformance", "Performance")
        DisabledTile {
            actionKey: "rotary-performance"
            iconName: "tools-rotary-performance"
            text: qsTranslate("RibbonActionRotaryPerformance", "Rotary\nPerformance Tool")
        }
    }
    Item {
        width: Theme.ribbonTrailingWidth
        height: Theme.ribbonHeight - Theme.borderWidth
    }
}

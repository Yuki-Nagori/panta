// Ribbon 公共外壳：保持背景与滚动规则，只实例化当前页签。
pragma ComponentBehavior: Bound

import QtQuick

Rectangle {
    id: panel
    objectName: "ribbonPanel"

    property string activeRibbonTab: "start-learn"
    signal newProjectRequested
    signal openProjectRequested
    signal importRequested

    implicitHeight: Theme.ribbonHeight
    gradient: Gradient {
        GradientStop {
            position: 0
            color: Theme.colorRibbonTop
        }
        GradientStop {
            position: 1
            color: Theme.colorRibbonBottom
        }
    }

    HorizontalToolStrip {
        id: ribbonStrip
        objectName: "ribbonStrip"
        anchors.fill: parent
        anchors.bottomMargin: Theme.borderWidth
        contentRoot: ribbonLoader

        Loader {
            id: ribbonLoader
            objectName: "ribbonLoader"
            height: ribbonStrip.height
            // 不约束宽度，由页签内容决定滚动范围；Loader 的 focus scope 放行工具键盘焦点。
            focus: true
            sourceComponent: panel.activeRibbonTab === "home" ? homeComponent : startLearnComponent
        }
    }

    Component {
        id: homeComponent
        HomeRibbon {
            onImportRequested: panel.importRequested()
        }
    }

    Component {
        id: startLearnComponent
        StartLearnRibbon {
            onNewProjectRequested: panel.newProjectRequested()
            onOpenProjectRequested: panel.openProjectRequested()
        }
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: Theme.borderWidth
        color: Theme.colorChromeLine
    }
}

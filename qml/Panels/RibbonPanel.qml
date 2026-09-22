// Ribbon 启动区（复刻件 .ribbon）：灰渐变条带承载启动/新功能/学习磁贴；
// 本任务为视觉骨架，磁贴暂未接业务命令。
import QtQuick

Rectangle {
    id: ribbon

    implicitWidth: 600
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

    Row {
        anchors.fill: parent
        anchors.margins: Theme.spacingTiny
        // 分隔线独占底边，不能侵占磁贴下方的可见留白。
        anchors.bottomMargin: Theme.spacingTiny + Theme.borderWidth
        spacing: Theme.spacingTiny

        RibbonTile {
            height: parent.height
            objectName: "ribbonStart"
            iconName: "ribbon-start"
            text: qsTr("Start")
        }
        RibbonTile {
            height: parent.height
            objectName: "ribbonNew"
            iconName: "ribbon-whatsnew"
            text: qsTr("New")
        }
        RibbonTile {
            height: parent.height
            objectName: "ribbonLearn"
            iconName: "ribbon-learn"
            text: qsTr("Learning")
        }
    }

    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: Theme.borderWidth
        color: Theme.colorChromeLine
    }
}

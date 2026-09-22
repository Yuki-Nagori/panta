// 页签条组合组件（复刻件 .panel-tabs/.pane-tabs）：左面板页签与 VTK 底部
// 视图页签共用样式；edge 决定分隔线方位与页签开口方向，激活页签白底加粗，
// 未激活灰底弱化。选中状态由 TabBar 的互斥 checked 语义承接。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls

TabBar {
    id: bar

    property alias tabs: repeater.model
    property int edge: Qt.TopEdge

    implicitHeight: Theme.controlHeight

    background: Rectangle {
        color: Theme.colorChrome

        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: Theme.borderWidth
            visible: bar.edge === Qt.TopEdge
            color: Theme.colorPanelLine
        }
        Rectangle {
            anchors.top: parent.top
            width: parent.width
            height: Theme.borderWidth
            visible: bar.edge === Qt.BottomEdge
            color: Theme.colorChromeLine
        }
    }

    Repeater {
        id: repeater

        delegate: TabButton {
            id: tab

            required property string modelData

            text: tab.modelData
            width: tab.implicitWidth
            leftPadding: Theme.spacingLarge
            rightPadding: Theme.spacingLarge
            topPadding: Theme.spacingXSmall
            bottomPadding: Theme.spacingXSmall
            font.weight: tab.checked ? Font.DemiBold : Font.Normal

            background: Rectangle {
                color: tab.checked ? Theme.colorPanel : Theme.colorChrome
                topLeftRadius: bar.edge === Qt.TopEdge ? Theme.radiusLarge : 0
                topRightRadius: bar.edge === Qt.TopEdge ? Theme.radiusLarge : 0
                bottomLeftRadius: bar.edge === Qt.BottomEdge ? Theme.radiusLarge : 0
                bottomRightRadius: bar.edge === Qt.BottomEdge ? Theme.radiusLarge : 0
                border.width: Theme.borderWidth
                border.color: Theme.colorPanelLine
            }

            contentItem: ThemedLabel {
                text: tab.text
                textSize: Theme.fontSmall
                textColor: tab.checked ? Theme.colorText : Theme.colorTextMuted
            }
        }
    }
}

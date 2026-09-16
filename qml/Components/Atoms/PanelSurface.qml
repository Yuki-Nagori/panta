// 面板表面原子：只负责颜色和几何外观，不依赖业务状态。
import QtQuick

Rectangle {
    id: surface

    property color surfaceColor: Theme.colorPanel
    property real cornerRadius: Theme.radiusSmall

    color: surfaceColor
    radius: cornerRadius
}

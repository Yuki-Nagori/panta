// 面板表面原子：只负责表面颜色这一外观职责，不依赖业务状态；
// 复刻件中面板为直角白色表面，圆角由控件层自行处理。
import QtQuick

Rectangle {
    id: surface

    property color surfaceColor: Theme.colorPanel

    color: surfaceColor
}

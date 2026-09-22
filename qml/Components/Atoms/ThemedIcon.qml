// 图标原子：渲染模块内置的 SVG 图标资源（qml/icons/，经 qt_add_resources
// 登记，映射自 050 复刻件的内联 SVG 占位）；只负责按 token 尺寸取图，
// 不携带动作语义。颜色固定在资源内，主题化图标资源由后续任务替换。
import QtQuick

Image {
    id: icon

    property string name: ""
    property int iconSize: Theme.iconSizeDefault

    width: iconSize
    height: iconSize
    source: icon.name !== "" ? Qt.resolvedUrl("../../icons/" + icon.name + ".svg") : ""
    sourceSize: Qt.size(iconSize, iconSize)
    fillMode: Image.PreserveAspectFit
    antialiasing: true
}

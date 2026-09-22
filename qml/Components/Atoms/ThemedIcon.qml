// 装饰性 Mono 图标：资源只描述几何，provider 按调用方颜色渲染；动作语义由宿主承担。
import QtQuick

Image {
    id: glyph

    property string name: ""
    property int iconSize: Theme.iconSizeDefault
    property color color: Theme.colorIcon

    // 固定八位 ARGB，避免 color 字符串省略不透明 alpha 或 URL 中的 # 片段。
    readonly property string colorKey: [color.a, color.r, color.g, color.b].map(channel => Math.round(channel * 255).toString(16).padStart(2, "0")).join("")

    width: iconSize
    height: iconSize
    source: name !== "" ? "image://panta-icons/" + name + "/" + colorKey : ""
    sourceSize: Qt.size(iconSize, iconSize)
    fillMode: Image.PreserveAspectFit
    Accessible.ignored: true
}

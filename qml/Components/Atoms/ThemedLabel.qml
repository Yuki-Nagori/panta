// 可复用文本原子：保留 Qt Controls Label 的焦点/可访问性语义。
import QtQuick
import QtQuick.Controls

Label {
    id: label

    property color textColor: Theme.colorText
    property int textSize: Theme.fontBody

    color: textColor
    font.pixelSize: textSize
}

// 可复用按钮原子：尺寸由 Theme 提供，也允许组合组件显式覆盖。
import QtQuick
import QtQuick.Controls

Button {
    id: button

    property int controlHeight: Theme.controlHeight
    property int contentPadding: Theme.spacingMedium

    implicitHeight: controlHeight
    leftPadding: contentPadding
    rightPadding: contentPadding
}

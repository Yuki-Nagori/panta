// 工艺弹窗共用控件；内联类型跨文件使用，不绑定到此文件的创建上下文。
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

QtObject {
    component Action: ThemedToolButton {
        Keys.onReturnPressed: event => {
            clicked();
            event.accepted = true;
        }
        contentColor: Theme.colorText
        borderColor: Theme.colorPanelLine
        contentPadding: Theme.spacingLarge
        controlHeight: Theme.controlHeight + Theme.spacingSmall
    }
    component Section: GroupBox {
        id: section
        Layout.fillWidth: true
        padding: Theme.spacingLarge
        topPadding: label.implicitHeight + Theme.spacingLarge
        label: ThemedLabel {
            text: section.title
            font.weight: Font.DemiBold
            leftPadding: Theme.spacingSmall
            rightPadding: Theme.spacingSmall
            background: Rectangle {
                color: Theme.colorPanel
            }
        }
        background: Rectangle {
            y: section.label.implicitHeight / 2
            height: section.height - y
            color: Theme.colorPanel
            border.color: Theme.colorPanelLine
            border.width: Theme.borderWidth
            radius: Theme.radiusSmall
        }
    }
    component ValueField: ThemedTextField {
        Layout.fillWidth: true
        Layout.preferredHeight: Theme.controlHeight + Theme.spacingSmall
        selectByMouse: true
        invalid: text.length > 0 && !acceptableInput
        validator: DoubleValidator {
            locale: "C"
            notation: DoubleValidator.StandardNotation
        }
    }
    component Choice: ComboBox {
        id: choice
        Layout.fillWidth: true
        implicitHeight: Theme.controlHeight + Theme.spacingSmall
        leftPadding: Theme.spacingMedium
        rightPadding: Theme.iconSizeSmall + 2 * Theme.spacingSmall
        contentItem: ThemedLabel {
            text: choice.displayText
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        indicator: ThemedIcon {
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            name: "caret"
            iconSize: Theme.iconSizeSmall
        }
        background: Rectangle {
            color: Theme.colorPanel
            border.width: choice.visualFocus ? Theme.focusBorderWidth : Theme.borderWidth
            border.color: choice.visualFocus ? Theme.colorFocus : Theme.colorPanelLine
            radius: Theme.radiusSmall
        }
    }
}

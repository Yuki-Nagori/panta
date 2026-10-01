// 099 公共控件的手动基准场景；窗口结构由既有弹窗场景覆盖。
import QtQuick
import QtQuick.Layouts
import Panta.Shell

Item {
    id: scene
    property int itemCount: 1
    ColumnLayout {
        anchors.fill: parent
        FormSection {
            Layout.fillWidth: true
            title: "Controls"
            ColumnLayout {
                width: parent.width
                Repeater {
                    model: scene.itemCount
                    delegate: RowLayout {
                        MenuTabButton {
                            text: "Results"
                        }
                        ThemedRadioButton {
                            text: "Mode"
                            checked: true
                        }
                        ThemedCheckBox {
                            text: "Log"
                            checked: true
                        }
                        ThemedComboBox {
                            model: ["One", "Two", "Three"]
                            font.pixelSize: Theme.fontBody
                        }
                        ThemedNumberField {
                            Layout.fillWidth: true
                            text: "40"
                        }
                        ThemedButton {
                            contentPadding: Theme.spacingLarge
                            text: "Edit"
                        }
                    }
                }
            }
        }
        Item {
            Layout.fillHeight: true
        }
        DialogButtonRow {
            Layout.fillWidth: true
            ThemedButton {
                text: "OK"
                primaryAction: true
            }
            ThemedButton {
                text: "Cancel"
            }
        }
    }
}

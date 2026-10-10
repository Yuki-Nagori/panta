// 业务动作由宿主注入；未配置动作的按钮统一请求占位提示。
import QtQuick
import QtQuick.Controls

ToolButton {
    property var clickAction: null
    onClicked: {
        if (clickAction)
            clickAction();
        else
            FeatureNotice.notify(Window.window);
    }
}

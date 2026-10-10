// 占位动作只报告当前窗口；Shell 统一承接原生提示窗口。
pragma Singleton
import QtQuick

QtObject {
    signal requested(var ownerWindow)
    function notify(ownerWindow) {
        requested(ownerWindow);
    }
}

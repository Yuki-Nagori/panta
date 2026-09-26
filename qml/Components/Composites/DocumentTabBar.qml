// 浏览器式视口文档页签（080）：130px 固定宽、可关闭、水平拖拽重排。
// 交互契约移植自 ai-docs/qml-html：4px 启动阈值、垂直手势取消、拖动
// 非活动就绪页签立即激活、中点换位 + 150ms 让位动画、拖动关闭按钮不
// 触发重排；系统减少动态效果偏好下跳过位移动画。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls

Item {
    id: bar

    implicitWidth: 200
    implicitHeight: Theme.documentTabBarHeight

    // ViewModel 投影：[{id, kind, state, title, message}]；顺序即页签顺序。
    property var documents: []
    property string activeDocumentId: ""
    // 系统减少动态效果偏好；true 时不播让位 / 归位动画。
    property bool reducedMotion: false

    signal activateDocument(string documentId)
    signal closeDocument(string documentId)
    signal moveDocument(int fromIndex, int toIndex)

    clip: true

    // 本地预览顺序：拖拽期间只改它并让邻居动画让位，松手一次性提交
    // moveDocument，避免模型重建打断手势。非拖拽时与 documents 同步。
    property var order: []
    property var itemRegistry: ({})
    property string draggedId: ""
    // 拖拽中页签的 x（bar 坐标）；槽位绑定它实现指针跟随。
    property real dragX: 0
    readonly property bool dragging: draggedId !== ""
    onDocumentsChanged: {
        if (dragging) {
            // 拖拽中模型被外部改变（工程切换等）：放弃手势并回同步。
            draggedId = "";
            dragSurface.enabled = false;
        }
        sync_order();
    }

    function sync_order() {
        const ids = [];
        for (let i = 0; i < documents.length; ++i) {
            ids.push(documents[i].id);
        }
        order = ids;
    }
    function slot_x(index) {
        return Theme.spacingTiny + index * (Theme.documentTabWidth + Theme.spacingTiny);
    }
    function slot_of(id) {
        return order.indexOf(id);
    }

    Component.onCompleted: sync_order()

    Rectangle {
        anchors.fill: parent
        color: Theme.colorDocumentBand
    }
    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: Theme.borderWidth
        color: Theme.colorDocumentBandLine
    }
    // 带下缘连续白线：活动页签的圆弧件覆盖其两侧，交界即圆弧过渡。
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 3
        color: "#ffffff"
    }

    // 拖拽捕获面：覆盖整条标签带，仅在拖拽启动后启用；普通点击仍由
    // 页签自身 MouseArea 完成目标判定。
    MouseArea {
        id: dragSurface
        anchors.fill: parent
        enabled: false
        cursorShape: Qt.ClosedHandCursor
        // 捕获面与 bar 同域，mouse.x 即 bar 坐标。
        onPositionChanged: mouse => bar.drag_move(mouse.x)
        onReleased: bar.finish_drag()
        onCanceled: bar.finish_drag()
    }

    Flickable {
        id: scroller
        anchors.fill: parent
        contentWidth: 2 * Theme.spacingTiny + bar.documents.length * (Theme.documentTabWidth + Theme.spacingTiny)
        clip: false
        interactive: contentWidth > width && !bar.dragging
        boundsBehavior: Flickable.StopAtBounds

        Repeater {
            id: tabRepeater
            model: bar.documents.length

            Item {
                id: tab
                required property int index

                readonly property var doc: bar.documents[index] ?? {}
                readonly property string documentId: doc.id ?? ""
                readonly property string tabState: doc.state ?? "ready"
                readonly property bool isActive: documentId === bar.activeDocumentId
                readonly property bool isDragged: documentId === bar.draggedId
                Component.onCompleted: bar.itemRegistry[documentId] = this
                // 关闭中间页签时 Repeater 按索引收缩，幸存委托的
                // documentId 会变化，映射必须同步刷新。
                onDocumentIdChanged: if (documentId !== "")
                    bar.itemRegistry[documentId] = this
                Component.onDestruction: delete bar.itemRegistry[documentId]
                // 拖拽中页签脱离槽位跟随指针；其余页签绑定槽位并动画让位。
                y: 2
                x: isDragged ? bar.dragX : bar.slot_x(bar.slot_of(documentId))
                Behavior on x {
                    enabled: !tab.isDragged && !bar.reducedMotion
                    NumberAnimation {
                        duration: Theme.documentTabAnimationDuration
                        easing.type: Easing.OutQuad
                    }
                }

                width: Theme.documentTabWidth
                height: Theme.documentTabBarHeight - 2 - 3
                z: tab.isActive ? 1 : 0

                Rectangle {
                    anchors.fill: parent
                    radius: Theme.documentTabRadius
                    color: tab.isActive ? Theme.colorPanel : tabArea.containsMouse ? Theme.colorDocumentHover : "transparent"
                }
                // 左右凹弧连接件：白色矩形带顶部外侧圆角，标签带灰从
                // 圆角切口透出——白线经凹弧平滑上弯进入页签（HTML 参考
                // 的浏览器式过渡）。随拖拽同步移动。
                Rectangle {
                    visible: tab.isActive
                    x: -Theme.documentTabRadius
                    y: tab.height - Theme.documentTabRadius
                    width: Theme.documentTabRadius
                    height: Theme.documentTabRadius
                    topLeftRadius: Theme.documentTabRadius
                    color: Theme.colorPanel
                }
                Rectangle {
                    visible: tab.isActive
                    x: tab.width
                    y: tab.height - Theme.documentTabRadius
                    width: Theme.documentTabRadius
                    height: Theme.documentTabRadius
                    topRightRadius: Theme.documentTabRadius
                    color: Theme.colorPanel
                }

                MouseArea {
                    id: tabArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: bar.dragging ? Qt.ClosedHandCursor : Qt.OpenHandCursor
                    acceptedButtons: Qt.LeftButton
                    onPressed: mouse => bar.begin_press(tab.documentId, mouse.x, mouse.y)
                    onPositionChanged: mouse => bar.detect_drag(tab.documentId, mouse)
                    onClicked: mouse => {
                        if (!internal.suppressClick) {
                            bar.activateDocument(tab.documentId);
                        }
                    }
                }

                Item {
                    anchors.left: parent.left
                    anchors.leftMargin: 11
                    anchors.right: closeButton.left
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    height: parent.height
                    clip: true

                    Row {
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: Theme.spacingMedium

                        Rectangle {
                            visible: (tab.doc.kind ?? "") === "welcome"
                            width: 17
                            height: 17
                            radius: 4
                            // HTML 参考的砖红（#a83e47），非品牌红。
                            color: "#a83e47"
                            ThemedLabel {
                                anchors.centerIn: parent
                                text: "P"
                                textSize: 11
                            }
                        }
                        BusyIndicator {
                            visible: tab.tabState === "loading"
                            width: 16
                            height: 16
                            running: visible
                        }
                        ThemedIcon {
                            visible: (tab.doc.kind ?? "") !== "welcome" && tab.tabState !== "loading"
                            name: "mesh"
                            iconSize: 16
                            color: tab.tabState === "failed" ? Theme.colorError : Theme.colorIcon
                        }
                        ThemedLabel {
                            width: Math.min(implicitWidth, 66)
                            text: tab.doc.title ?? ""
                            textSize: Theme.fontBody
                            color: tab.tabState === "failed" ? Theme.colorError : tab.isActive ? Theme.colorText : Theme.colorTextMuted
                            elide: Text.ElideRight
                        }
                    }
                }

                Rectangle {
                    id: closeButton
                    objectName: "documentTabClose"
                    anchors.right: parent.right
                    anchors.rightMargin: 3
                    anchors.verticalCenter: parent.verticalCenter
                    width: Theme.documentTabCloseSize
                    height: Theme.documentTabCloseSize
                    radius: 5
                    color: closeArea.containsMouse ? Theme.colorHover : "transparent"

                    ThemedLabel {
                        anchors.centerIn: parent
                        text: "×"
                        textSize: 12
                    }
                    MouseArea {
                        id: closeArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: bar.closeDocument(tab.documentId)
                    }
                }
            }
        }
    }

    QtObject {
        id: internal
        property real pressX: 0
        property real pressY: 0
        property string pressedId: ""
        property real grabOffset: 0
        property bool suppressClick: false
    }

    function begin_press(documentId, x, y) {
        internal.pressedId = documentId;
        internal.pressX = x;
        internal.pressY = y;
        internal.suppressClick = false;
    }
    function detect_drag(documentId, mouse) {
        if (internal.pressedId !== documentId || bar.dragging) {
            return;
        }
        const dx = mouse.x - internal.pressX;
        const dy = mouse.y - internal.pressY;
        if (Math.max(Math.abs(dx), Math.abs(dy)) < Theme.documentTabDragThreshold) {
            return;
        }
        if (Math.abs(dx) <= Math.abs(dy)) {
            internal.pressedId = "";
            return;
        }
        // 拖拽启动：非活动就绪页签立即激活；捕获面接管后续移动，
        // 抑制回到起点时的误点击。
        internal.suppressClick = true;
        bar.activateDocument(documentId);
        bar.draggedId = documentId;
        const item = tab_item_at(bar.slot_of(documentId));
        internal.grabOffset = item ? bar.mapFromItem(item, mouse.x, mouse.y).x : 0;
        bar.dragX = bar.mapFromItem(item, 0, 0).x;
        dragSurface.enabled = true;
        dragSurface.grabMouse(); // qmllint disable missing-property
    }
    function drag_move(pointerX) {
        if (!bar.dragging) {
            return;
        }
        bar.dragX = pointerX - internal.grabOffset;
        const draggedIndex = order.indexOf(bar.draggedId);
        if (draggedIndex < 0) {
            return;
        }
        const slotSpan = Theme.documentTabWidth + Theme.spacingTiny;
        const pointerSlot = Math.round((pointerX - Theme.spacingTiny) / slotSpan - 0.5);
        const target = pointerX < slot_x(pointerSlot) + Theme.documentTabWidth / 2 ? pointerSlot : pointerSlot + 1;
        const clamped = Math.max(0, Math.min(order.length - 1, target));
        if (clamped === draggedIndex) {
            return;
        }
        const next = order.slice();
        next.splice(draggedIndex, 1);
        next.splice(clamped, 0, bar.draggedId);
        order = next;
    }
    function finish_drag() {
        if (!bar.dragging) {
            return;
        }
        const from = bar.documents.findIndex(entry => entry.id === bar.draggedId);
        const to = order.indexOf(bar.draggedId);
        bar.draggedId = "";
        sync_order();
        if (from >= 0 && to >= 0 && from !== to) {
            moveDocument(from, to);
        }
    }
    // 委托注册表：documentId → 条目引用；qmllint 无法内联解析
    // Repeater.itemAt 的动态委托类型，注册表同时避免 O(n) 查找。
    function tab_item_at(documentId) {
        return itemRegistry[documentId] ?? null;
    }
}

// 浏览器式视口文档页签（080）：130px 固定宽、可关闭、水平拖拽重排。
// 交互契约移植自 ai-docs/qml-html：4px 启动阈值、垂直手势取消、拖动
// 非活动就绪页签立即激活、中点换位 + 150ms 让位动画、拖动关闭按钮不
// 触发重排；系统减少动态效果偏好下跳过位移动画。
pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Shapes

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
    property string draggedId: ""
    // 拖拽中页签的 x（bar 坐标）；槽位绑定它实现指针跟随。
    property real dragX: 0
    readonly property bool dragging: draggedId !== ""
    onDocumentsChanged: {
        if (dragging) {
            // 拖拽中模型被外部改变（工程切换等）：放弃手势并回同步。
            draggedId = "";
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

    // 重排位移差动画（对应 HTML 的 animateTabX）：order 变化后，把每个
    // 委托「当前视觉位置 − 新槽位」记为 visualOffset，再由 Behavior 平滑
    // 归零——与 HTML 的 insertBefore + 差值动画完全同构。
    onOrderChanged: {
        for (let i = 0; i < tabRepeater.count; ++i) {
            const item = tabRepeater.itemAt(i);
            // itemAt 返回 QQuickItem*，qmllint 无法解析 Repeater 委托的
            // 动态属性（isDragged / documentId / visualOffset 运行时存在）。
            if (!item || item.isDragged) { // qmllint disable missing-property
                continue;
            }
            const target = slot_x(slot_of(item.documentId)); // qmllint disable missing-property
            item.visualOffset = item.x - target; // qmllint disable missing-property
        }
    }

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
                // 让位 / 归位位移差：order 重排后由 bar 层写入「当前视觉位置
                // − 新槽位」，Behavior 平滑归零（HTML animateTabX 的同构实现）。
                property real visualOffset: 0
                Behavior on visualOffset {
                    enabled: !tab.isDragged && !bar.reducedMotion
                    NumberAnimation {
                        duration: Theme.documentTabAnimationDuration
                        easing.type: Easing.OutQuad
                    }
                }
                // 拖拽中页签脱离槽位跟随指针；其余页签绑定槽位 + 位移差。
                y: 2
                x: isDragged ? bar.dragX : bar.slot_x(bar.slot_of(documentId)) + visualOffset

                width: Theme.documentTabWidth
                height: Theme.documentTabBarHeight - 2 - 3
                // 拖拽或让位 / 归位动画期间保持置顶，避免沉入邻页签下方。
                z: tab.isDragged || Math.abs(visualOffset) > 0.5 ? 2 : tab.isActive ? 1 : 0

                Rectangle {
                    anchors.fill: parent
                    topLeftRadius: Theme.documentTabRadius
                    topRightRadius: Theme.documentTabRadius
                    color: tab.isActive ? Theme.colorPanel : tabArea.containsMouse ? Theme.colorDocumentHover : "transparent"
                }
                // 左右凹弧连接件：QML 无伪元素与 box-shadow，用 Shape
                // 画内凹弧——面板色填充弧与右/下边之间的区域，标签带灰
                // 从弧外左上透出，白线经凹弧平滑上弯进入页签。随拖拽
                // 同步移动。
                Shape {
                    visible: tab.isActive
                    x: -Theme.documentTabRadius
                    y: tab.height - Theme.documentTabRadius
                    width: Theme.documentTabRadius
                    height: Theme.documentTabRadius
                    z: 2
                    ShapePath {
                        fillColor: Theme.colorPanel
                        strokeColor: "transparent"
                        strokeWidth: 0
                        startX: Theme.documentTabRadius
                        startY: 0
                        PathCubic {
                            x: 0
                            y: Theme.documentTabRadius
                            control1X: Theme.documentTabRadius
                            control1Y: Theme.documentTabRadius * 0.5522847498
                            control2X: Theme.documentTabRadius * 0.5522847498
                            control2Y: Theme.documentTabRadius
                        }
                        PathLine {
                            x: Theme.documentTabRadius
                            y: Theme.documentTabRadius
                        }
                        PathLine {
                            x: Theme.documentTabRadius
                            y: 0
                        }
                    }
                }
                Shape {
                    visible: tab.isActive
                    x: tab.width
                    y: tab.height - Theme.documentTabRadius
                    width: Theme.documentTabRadius
                    height: Theme.documentTabRadius
                    z: 2
                    ShapePath {
                        fillColor: Theme.colorPanel
                        strokeColor: "transparent"
                        strokeWidth: 0
                        startX: 0
                        startY: 0
                        PathCubic {
                            x: Theme.documentTabRadius
                            y: Theme.documentTabRadius
                            control1X: Theme.documentTabRadius * 0.5522847498
                            control1Y: 0
                            control2X: Theme.documentTabRadius
                            control2Y: Theme.documentTabRadius * 0.5522847498
                        }
                        PathLine {
                            x: 0
                            y: Theme.documentTabRadius
                        }
                        PathLine {
                            x: 0
                            y: 0
                        }
                    }
                }

                MouseArea {
                    id: tabArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: bar.dragging ? Qt.ClosedHandCursor : Qt.OpenHandCursor
                    acceptedButtons: Qt.LeftButton
                    // 拖拽一经启动不容 Flickable 抢抓，否则移动事件中断、
                    // 页签表现为凭空消失；标签带滚动由页签外空白区承担。
                    preventStealing: true
                    onPressed: mouse => {
                        const p = bar.mapFromItem(tab, mouse.x, mouse.y);
                        bar.begin_press(tab.documentId, p.x, p.y);
                    }
                    // 按下期间 MouseArea 隐式抓取指针，移出边界仍持续收到
                    // 移动；坐标换算到 bar 域后驱动阈值判定与拖拽跟随。
                    onPositionChanged: mouse => {
                        const p = bar.mapFromItem(tab, mouse.x, mouse.y);
                        if (!bar.dragging) {
                            bar.detect_drag(tab.documentId, p.x, p.y);
                        } else {
                            bar.drag_move(p.x);
                        }
                    }
                    onReleased: bar.finish_drag()
                    onCanceled: bar.finish_drag()
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

                    // 关闭图形复用全局 pane-close SVG，矢量居中不依赖字体。
                    ThemedIcon {
                        anchors.centerIn: parent
                        name: "pane-close"
                        iconSize: 12
                        color: closeArea.containsMouse ? Theme.colorText : Theme.colorTextMuted
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
    function detect_drag(documentId, pointerBarX, pressBarY) {
        if (internal.pressedId !== documentId || bar.dragging) {
            return;
        }
        const dx = pointerBarX - internal.pressX;
        const dy = pressBarY - internal.pressY;
        if (Math.max(Math.abs(dx), Math.abs(dy)) < Theme.documentTabDragThreshold) {
            return;
        }
        if (Math.abs(dx) <= Math.abs(dy)) {
            internal.pressedId = "";
            return;
        }
        // 拖拽启动：非活动就绪页签立即激活；grabOffset 记录指针相对槽位
        // 起点的偏移，使页签全程跟随指针不跳变；suppressClick 抑制松手
        // 回到起点时的误点击。
        internal.suppressClick = true;
        bar.activateDocument(documentId);
        bar.draggedId = documentId;
        internal.grabOffset = pointerBarX - slot_x(slot_of(documentId));
        bar.dragX = pointerBarX;
    }
    function drag_move(pointerX) {
        if (!bar.dragging) {
            return;
        }
        // 钳制在标签带内：越界时页签贴边停留（重排判定仍用原始指针位）。
        bar.dragX = Math.max(0, Math.min(pointerX - internal.grabOffset, bar.width - Theme.documentTabWidth));
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
        const releaseX = bar.dragX;
        const releasedId = bar.draggedId;
        bar.draggedId = "";
        sync_order();
        // 松手归位：给被拖委托写入「松手位置 − 新槽位」的位移差，由
        // Behavior 平滑归零（reducedMotion 时直接就位）。
        for (let i = 0; i < tabRepeater.count; ++i) {
            const item = tabRepeater.itemAt(i); // qmllint disable missing-property
            if (item !== null && item.documentId === releasedId) { // qmllint disable missing-property
                item.visualOffset = releaseX - slot_x(to); // qmllint disable missing-property
                break;
            }
        }
        if (from >= 0 && to >= 0 && from !== to) {
            moveDocument(from, to);
        }
    }
}

// 保压曲线编辑草稿；只有确认才把完整行交回父弹窗，空行不构成曲线点。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: dialog
    objectName: "holdingProfileDialog"
    property Window ownerWindow
    property bool plotVisible: false
    property int editRevision: 0
    readonly property var points: {
        // ListModel 的字段变更不会自动重算 JS 循环，显式用编辑版本触发投影。
        const revision = editRevision;
        const result = [];
        for (let i = 0; i < rows.count; ++i) {
            const row = rows.get(i);
            if (row.durationText === "" && row.pressureText === "")
                continue;
            const duration = Number(row.durationText);
            const pressure = Number(row.pressureText);
            if (row.durationText === "" || row.pressureText === "" || !Number.isFinite(duration) || !Number.isFinite(pressure) || duration < 0 || pressure < 0 || pressure > 200)
                return [];
            result.push({
                duration: duration,
                pressure: pressure
            });
        }
        return result;
    }
    signal profileAccepted(var points)
    width: Theme.holdingProfileDialogWidth
    height: Theme.holdingProfileDialogHeight
    minimumWidth: Theme.holdingProfileDialogMinimumWidth
    minimumHeight: Theme.holdingProfileDialogMinimumHeight
    color: Theme.colorTransparent
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: qsTranslate("HoldingProfile", "Pack/Holding Control Profile")
    transientParent: ownerWindow

    function open(profile) {
        rows.clear();
        for (const point of profile)
            rows.append({
                durationText: String(point.duration),
                pressureText: String(point.pressure)
            });
        while (rows.count < 3)
            rows.append({
                durationText: "",
                pressureText: ""
            });
        editRevision++;
        plotVisible = false;
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        show();
        requestActivate();
        table.forceActiveFocus();
    }
    function acceptProfile() {
        if (!visible || points.length === 0)
            return;
        profileAccepted(points);
        close();
    }
    onPointsChanged: plot.requestPaint()
    ListModel {
        id: rows
    }
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
    Rectangle {
        anchors.fill: parent
        color: Theme.colorPanel
        ColumnLayout {
            anchors.fill: parent
            spacing: 0
            DialogTitleBar {
                Layout.fillWidth: true
                window: dialog
                caption: dialog.title
                onCloseRequested: dialog.close()
            }
            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.margins: Theme.spacingLarge
                spacing: Theme.spacingLarge
                ThemedLabel {
                    text: qsTranslate("FillSettings", "Filling pressure vs time (%)")
                    textColor: Theme.colorTextMuted
                }
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: Theme.colorPanel
                    border.color: Theme.colorPanelLine
                    border.width: Theme.borderWidth
                    radius: Theme.radiusSmall
                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.borderWidth
                        spacing: 0
                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: header.implicitHeight + 2 * Theme.spacingMedium
                            color: Theme.colorChrome
                            RowLayout {
                                id: header
                                anchors.fill: parent
                                anchors.margins: Theme.spacingMedium
                                spacing: Theme.spacingMedium
                                ThemedLabel {
                                    Layout.preferredWidth: Theme.profileStepColumnWidth
                                    text: qsTranslate("HoldingProfile", "Step")
                                }
                                ThemedLabel {
                                    Layout.fillWidth: true
                                    Layout.preferredWidth: 0
                                    text: qsTranslate("HoldingProfile", "Duration") + "\n" + qsTranslate("HoldingProfile", "s · ≥ 0")
                                }
                                ThemedLabel {
                                    Layout.fillWidth: true
                                    Layout.preferredWidth: 0
                                    text: qsTranslate("HoldingProfile", "Filling pressure") + "\n% · 0–200"
                                }
                                Item {
                                    Layout.preferredWidth: Theme.controlHeight
                                }
                            }
                        }
                        ScrollView {
                            id: table
                            objectName: "holdingProfileTable"
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            contentWidth: availableWidth
                            clip: true
                            ColumnLayout {
                                width: table.availableWidth
                                spacing: Theme.spacingSmall
                                Repeater {
                                    model: rows
                                    delegate: RowLayout {
                                        id: row
                                        required property int index
                                        required property string durationText
                                        required property string pressureText
                                        Layout.fillWidth: true
                                        Layout.margins: Theme.spacingMedium
                                        spacing: Theme.spacingMedium
                                        ThemedLabel {
                                            Layout.preferredWidth: Theme.profileStepColumnWidth
                                            text: row.index + 1
                                            textColor: Theme.colorTextMuted
                                        }
                                        ThemedTextField {
                                            objectName: "profileDuration" + row.index
                                            Layout.fillWidth: true
                                            Layout.preferredWidth: 0
                                            Layout.preferredHeight: Theme.controlHeight + Theme.spacingSmall
                                            text: row.durationText
                                            selectByMouse: true
                                            Accessible.name: qsTranslate("HoldingProfile", "Step %1 duration in seconds").arg(row.index + 1)
                                            validator: DoubleValidator {
                                                bottom: 0
                                                locale: "C"
                                                notation: DoubleValidator.StandardNotation
                                            }
                                            invalid: (text.length > 0 && !acceptableInput) || (text.length === 0 && row.pressureText.length > 0)
                                            onTextEdited: {
                                                rows.setProperty(row.index, "durationText", text);
                                                dialog.editRevision++;
                                            }
                                        }
                                        ThemedTextField {
                                            objectName: "profilePressure" + row.index
                                            Layout.fillWidth: true
                                            Layout.preferredWidth: 0
                                            Layout.preferredHeight: Theme.controlHeight + Theme.spacingSmall
                                            text: row.pressureText
                                            selectByMouse: true
                                            Accessible.name: qsTranslate("HoldingProfile", "Step %1 filling pressure in percent").arg(row.index + 1)
                                            validator: DoubleValidator {
                                                bottom: 0
                                                top: 200
                                                locale: "C"
                                                notation: DoubleValidator.StandardNotation
                                            }
                                            invalid: (text.length > 0 && !acceptableInput) || (text.length === 0 && row.durationText.length > 0)
                                            onTextEdited: {
                                                rows.setProperty(row.index, "pressureText", text);
                                                dialog.editRevision++;
                                            }
                                        }
                                        Action {
                                            Layout.preferredWidth: Theme.controlHeight
                                            text: "×"
                                            accessibleName: qsTranslate("HoldingProfile", "Remove step")
                                            enabled: rows.count > 3
                                            onClicked: {
                                                rows.remove(row.index);
                                                dialog.editRevision++;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                RowLayout {
                    Layout.fillWidth: true
                    Action {
                        text: qsTranslate("HoldingProfile", "Add Step")
                        onClicked: {
                            rows.append({
                                durationText: "",
                                pressureText: ""
                            });
                            dialog.editRevision++;
                        }
                    }
                    Action {
                        text: qsTranslate("HoldingProfile", "Import Profile…")
                        enabled: false
                    }
                    Item {
                        Layout.fillWidth: true
                    }
                    Action {
                        text: qsTranslate("HoldingProfile", "Plot Profile")
                        enabled: dialog.points.length > 0
                        onClicked: dialog.plotVisible = !dialog.plotVisible
                    }
                }
                Canvas {
                    id: plot
                    objectName: "holdingProfilePlot"
                    Layout.fillWidth: true
                    Layout.preferredHeight: Theme.profilePlotHeight
                    visible: dialog.plotVisible
                    onVisibleChanged: if (visible)
                        requestPaint()
                    onWidthChanged: requestPaint()
                    onHeightChanged: requestPaint()
                    onPaint: {
                        const ctx = getContext("2d");
                        ctx.clearRect(0, 0, width, height);
                        const inset = Theme.spacingLarge;
                        const total = dialog.points.reduce((sum, point) => sum + point.duration, 0);
                        ctx.strokeStyle = Theme.colorPanelLine;
                        ctx.lineWidth = Theme.borderWidth;
                        ctx.beginPath();
                        ctx.moveTo(inset, inset);
                        ctx.lineTo(inset, height - inset);
                        ctx.lineTo(width - inset, height - inset);
                        ctx.stroke();
                        ctx.strokeStyle = Theme.colorFocus;
                        ctx.lineWidth = Theme.focusBorderWidth;
                        ctx.beginPath();
                        let time = 0;
                        dialog.points.forEach((point, index) => {
                            time += point.duration;
                            const x = inset + time / (total || 1) * (width - 2 * inset);
                            const y = height - inset - point.pressure / 200 * (height - 2 * inset);
                            if (index === 0)
                                ctx.moveTo(x, y);
                            else
                                ctx.lineTo(x, y);
                        });
                        ctx.stroke();
                    }
                    Accessible.role: Accessible.Chart
                    Accessible.name: qsTranslate("HoldingProfile", "Pressure vs cumulative time")
                }
                RowLayout {
                    Layout.fillWidth: true
                    Item {
                        Layout.fillWidth: true
                    }
                    Action {
                        objectName: "holdingProfileAccept"
                        Layout.preferredWidth: Theme.tabSegmentWidth
                        controlHeight: Theme.controlHeight
                        text: qsTranslate("DialogAction", "OK")
                        enabled: dialog.points.length > 0
                        primaryAction: true
                        borderColor: Theme.colorDialogPrimaryBorder
                        onClicked: dialog.acceptProfile()
                    }
                    Action {
                        Layout.preferredWidth: Theme.tabSegmentWidth
                        controlHeight: Theme.controlHeight
                        objectName: "holdingProfileCancel"
                        text: qsTranslate("DialogAction", "Cancel")
                        onClicked: dialog.close()
                    }
                    Action {
                        Layout.preferredWidth: Theme.tabSegmentWidth
                        controlHeight: Theme.controlHeight
                        text: qsTranslate("UiCommonHelp", "Help")
                        enabled: false
                    }
                }
            }
        }
    }
    Shortcut {
        sequence: "Esc"
        enabled: dialog.visible
        onActivated: dialog.close()
    }
}
